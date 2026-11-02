//! Discovery of MTPLX servers that this app did not start.
//!
//! The MTPLX desktop app and the `mtplx` CLI can both leave a server running.
//! Those servers are just as real as the ones we spawn, so the GUI lists them
//! too — flagged as *external* because we do not own their process.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::system::memory;

/// A running MTPLX endpoint found on the loopback interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalServer {
    /// Port the server answers on.
    pub port: u16,
    /// Model id the server advertises.
    pub model_id: String,
    /// Context length the server advertises, when it reports one.
    pub context_length: Option<u64>,
    /// Resident memory held by the server process, in bytes.
    pub memory_bytes: u64,
    /// Weight footprint of the pack backing this server, in bytes.
    ///
    /// Resolved from the local model cache, because an external server does
    /// not expose its own memory plan.
    pub weights_bytes: Option<u64>,
}

/// A cached pack, used to resolve the weight footprint of an external server.
#[derive(Debug, Clone)]
pub struct PackSize {
    pub repo_id: String,
    pub size_bytes: u64,
}

/// The `GET /v1/models` payload, narrowed to the fields we use.
#[derive(Debug, Clone, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    max_context_length: Option<u64>,
    #[serde(default)]
    context_length: Option<u64>,
}

/// Probe a single port and describe the MTPLX server behind it, if any.
async fn probe(client: &reqwest::Client, port: u16) -> Option<(u16, String, Option<u64>)> {
    let url = format!("http://127.0.0.1:{port}/v1/models");
    let response = client.get(&url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let payload: ModelsResponse = response.json().await.ok()?;
    let entry = payload.data.into_iter().next()?;
    Some((
        port,
        entry.id,
        entry.max_context_length.or(entry.context_length),
    ))
}

/// Scan a port range for MTPLX servers.
///
/// Probes run concurrently so a closed port costs nothing but a refused
/// connection. Memory attribution runs afterwards, once per server found.
pub async fn scan(ports: &[u16], packs: &[PackSize]) -> Vec<ExternalServer> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(600))
        .connect_timeout(Duration::from_millis(300))
        .build()
    {
        Ok(client) => client,
        Err(_) => return Vec::new(),
    };

    let mut futures = Vec::with_capacity(ports.len());
    for &port in ports {
        futures.push(probe(&client, port));
    }

    let mut found = Vec::new();
    for result in futures::future::join_all(futures).await {
        if let Some((port, model_id, context_length)) = result {
            found.push(ExternalServer {
                port,
                model_id: model_id.clone(),
                context_length,
                memory_bytes: 0,
                weights_bytes: resolve_weights(&model_id, packs),
            });
        }
    }
    found.sort_by_key(|server| server.port);

    // Attribute memory after the probes, so the `sysinfo` handle is not
    // borrowed across the concurrent section.
    let mut sys = memory::new_system();
    for server in &mut found {
        server.memory_bytes = memory::memory_for_port(&mut sys, server.port);
    }

    found
}

/// Resolve the weight footprint of a running model from the local cache.
///
/// The server advertises a short model id (`mtplx-qwen38-27b-optimized-quality`)
/// while the cache is keyed by repo id (`Youssofal/Qwen3.8-27B-MTPLX-...`), so
/// the two are matched on their longest shared alphanumeric run.
pub fn resolve_weights(model_id: &str, packs: &[PackSize]) -> Option<u64> {
    let needle = normalize(model_id);
    let mut best: Option<(usize, u64)> = None;

    for pack in packs {
        let haystack = normalize(&pack.repo_id);
        let shared = longest_common_run(&needle, &haystack);
        if shared >= 8 {
            let better = best.map(|(len, _)| shared > len).unwrap_or(true);
            if better {
                best = Some((shared, pack.size_bytes));
            }
        }
    }

    best.map(|(_, bytes)| bytes)
}

/// Lower-case and drop everything that is not a letter or a digit.
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Length of the longest shared run of characters between two strings.
fn longest_common_run(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() || b.is_empty() {
        return 0;
    }

    let mut best = 0usize;
    let mut previous = vec![0usize; b.len() + 1];
    let mut current = vec![0usize; b.len() + 1];

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            current[j] = if a[i - 1] == b[j - 1] {
                let run = previous[j - 1] + 1;
                best = best.max(run);
                run
            } else {
                0
            };
        }
        std::mem::swap(&mut previous, &mut current);
    }

    best
}
