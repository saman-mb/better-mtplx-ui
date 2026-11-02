//! End-to-end check that two real MTPLX instances run in parallel.
//!
//! Run with: `cargo run --example parallel`
//!
//! Starts two real models through the orchestrator, waits for both to report
//! ready, sends a concurrent request to each, then stops both.

use std::sync::Arc;
use std::time::Duration;

use better_mtplx_ui_lib::instances::manager::InstanceManager;
use better_mtplx_ui_lib::instances::types::InstanceStatus;

const MODELS: [&str; 2] = [
    "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Quality",
    "Youssofal/Qwen3.8-Flash-Next-MTPLX-Optimized-Speed",
];
const PORTS: [u16; 2] = [8021, 8022];

#[tokio::main]
async fn main() {
    let manager = Arc::new(InstanceManager::new());
    let mut ids = Vec::new();

    for (model, port) in MODELS.iter().zip(PORTS.iter()) {
        println!("starting {model} on {port} …");
        let instance = manager
            .start(model, *port, &["--profile".into(), "turbo".into()])
            .await
            .expect("start instance");
        println!("  spawned id={}", instance.id);
        ids.push(instance.id);
    }

    // Wait for both to become ready.
    let mut ready = false;
    for _ in 0..240 {
        manager.refresh().await;
        let views = manager.views();
        if views.iter().all(|v| v.status == InstanceStatus::Ready) {
            ready = true;
            break;
        }
        if views.iter().any(|v| v.status == InstanceStatus::Failed) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("\n--- instance state ---");
    for view in manager.views() {
        println!(
            "port={} status={:?} resident={:.1} GB weights={:?}",
            view.port,
            view.status,
            view.memory_bytes as f64 / 1024f64.powi(3),
            view.weights_bytes
                .map(|w| format!("{:.1} GB", w as f64 / 1024f64.powi(3)))
        );
    }
    println!("both ready = {ready}");

    if ready {
        // Fire one request at each instance at the same time.
        let client = reqwest::Client::new();
        let mut tasks = Vec::new();
        for (index, port) in PORTS.iter().enumerate() {
            let client = client.clone();
            let port = *port;
            let model = MODELS[index];
            tasks.push(tokio::spawn(async move {
                let body = serde_json::json!({
                    "model": model,
                    "messages": [{"role": "user", "content": format!("Reply with exactly PARALLEL-{port}")}],
                    "max_tokens": 48,
                    "stream": false,
                });
                let started = std::time::Instant::now();
                let response = client
                    .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
                    .json(&body)
                    .send()
                    .await;
                match response {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        let payload: serde_json::Value =
                            response.json().await.unwrap_or_default();
                        let content = payload["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        (
                            port,
                            status,
                            started.elapsed().as_secs_f64(),
                            content.chars().take(40).collect::<String>(),
                        )
                    }
                    Err(err) => (port, 0, started.elapsed().as_secs_f64(), err.to_string()),
                }
            }));
        }

        println!("\n--- concurrent requests ---");
        for task in tasks {
            let (port, status, elapsed, content) = task.await.expect("task");
            println!("port={port} status={status} elapsed={elapsed:.2}s {content:?}");
        }
    }

    println!("\nstopping both …");
    for id in &ids {
        let _ = manager.stop(id).await;
    }
    manager.stop_all().await;
    println!("done");
}
