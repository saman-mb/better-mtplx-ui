//! Locating and invoking the `mtplx` CLI.
//!
//! MTPLX ships its binary at `~/.mtplx/bin/mtplx` and puts it on `PATH` via
//! the app. We resolve it ourselves so the GUI works even when launched from
//! an app bundle with a minimal environment.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::process::Command;

use super::types::{DiscoverCard, InspectResult, ModelsPayload, StatusPayload};

/// Canonical install location of the MTPLX CLI.
pub const MTPLX_HOME: &str = "~/.mtplx";

/// Environment variable that overrides binary discovery.
///
/// Used by the test-suite to point the orchestrator at a stub, and handy for
/// anyone running a non-standard install.
pub const MTPLX_BIN_ENV: &str = "MTPLX_BIN";

/// Resolve the absolute path of the `mtplx` binary.
///
/// Order: the `MTPLX_BIN` override, the well-known install path, then `PATH`.
pub fn resolve_binary() -> Result<PathBuf> {
    if let Ok(explicit) = std::env::var(MTPLX_BIN_ENV) {
        let path = PathBuf::from(&explicit);
        if path.is_file() {
            return Ok(path);
        }
        bail!("{MTPLX_BIN_ENV} points at {explicit}, which is not a file")
    }

    let home = std::env::var("HOME").context("HOME is not set")?;
    let candidate = Path::new(&home).join(".mtplx").join("bin").join("mtplx");
    if candidate.is_file() {
        return Ok(candidate);
    }

    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            if dir.is_empty() {
                continue;
            }
            let candidate = Path::new(dir).join("mtplx");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    bail!("mtplx binary not found at ~/.mtplx/bin/mtplx or on PATH")
}

/// Run `mtplx` with the given arguments and capture stdout.
///
/// Returns the trimmed stdout as a string. Stderr is folded into the error
/// message so CLI failures are actionable in the UI.
pub async fn run(args: &[&str]) -> Result<String> {
    let bin = resolve_binary()?;
    let output = Command::new(&bin)
        .args(args)
        .stdin(Stdio::null())
        .kill_on_drop(false)
        .output()
        .await
        .with_context(|| format!("failed to execute {} {}", bin.display(), args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        bail!(
            "mtplx {} exited with {}: {}",
            args.join(" "),
            output.status,
            if detail.is_empty() {
                "no output".into()
            } else {
                detail
            }
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run `mtplx` and parse stdout as JSON.
pub async fn run_json<T: serde::de::DeserializeOwned>(args: &[&str]) -> Result<T> {
    let stdout = run(args).await?;
    serde_json::from_str(&stdout)
        .with_context(|| format!("failed to parse JSON from `mtplx {}`", args.join(" ")))
}

/// List cached model packs.
pub async fn list_models() -> Result<ModelsPayload> {
    run_json(&["models", "--json"]).await
}

/// Inspect a pack for runtime-contract / MTP compatibility.
pub async fn inspect(repo_id: &str, require_mtp: bool) -> Result<InspectResult> {
    let mut args = vec!["inspect", repo_id, "--json"];
    if require_mtp {
        args.push("--require-mtp");
    }
    run_json(&args).await
}

/// Fetch install / model / integration health.
pub async fn status() -> Result<StatusPayload> {
    run_json(&["status", "--json"]).await
}

/// Browse MTPLX-branded models published on Hugging Face.
///
/// `query` defaults to `MTPLX`, which is what the engine itself searches for.
pub async fn discover(query: Option<&str>, limit: u32) -> Result<Vec<DiscoverCard>> {
    let query = query.unwrap_or("MTPLX");
    let limit = limit.clamp(1, 100);
    let limit_arg = limit.to_string();
    run_json(&[
        "forge", "discover", "--json", "--query", query, "--limit", &limit_arg,
    ])
    .await
}

/// Start a model download in the background.
///
/// Returns as soon as the child is spawned; progress is reported by polling
/// `mtplx models --json` from the UI.
pub async fn pull(repo_id: &str) -> Result<()> {
    let bin = resolve_binary()?;
    Command::new(&bin)
        .args(["pull", repo_id])
        .stdin(Stdio::null())
        .kill_on_drop(false)
        .spawn()
        .with_context(|| format!("failed to spawn `mtplx pull {repo_id}`"))?;
    Ok(())
}
