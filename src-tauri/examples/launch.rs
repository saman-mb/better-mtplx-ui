//! End-to-end check of the orchestrator against the real MTPLX engine.
//!
//! Run with: `cargo run --example launch -- <model-repo> [port]`
//!
//! Starts a real instance through the same code path the GUI uses, waits for
//! it to report ready, prints its memory attribution, then stops it.

use std::sync::Arc;
use std::time::Duration;

use better_mtplx_ui_lib::instances::manager::InstanceManager;
use better_mtplx_ui_lib::instances::types::InstanceStatus;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let model = args
        .next()
        .unwrap_or_else(|| "Youssofal/Qwen3.8-27B-MTPLX-Optimized-Quality".to_string());
    let port: u16 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8020);

    let manager = Arc::new(InstanceManager::new());

    println!("starting {model} on port {port} …");
    let instance = manager
        .start(&model, port, &["--profile".into(), "turbo".into()])
        .await
        .expect("start instance");
    println!("spawned: id={}", instance.id);

    let mut ready = false;
    for _ in 0..180 {
        manager.refresh().await;
        let view = manager.view(&instance.id).expect("view");
        if view.status == InstanceStatus::Ready {
            ready = true;
            break;
        }
        if view.status == InstanceStatus::Failed {
            println!("FAILED: {:?}", view.error);
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let view = manager.view(&instance.id).expect("view");
    println!(
        "status={:?} memory={:.2} GB",
        view.status,
        view.memory_bytes as f64 / 1024f64.powi(3)
    );
    println!("ready={ready}");

    if ready {
        // Confirm the endpoint actually answers the OpenAI models route.
        let url = format!("http://127.0.0.1:{port}/v1/models");
        match reqwest::get(&url).await {
            Ok(response) => println!("GET /v1/models -> {}", response.status()),
            Err(err) => println!("GET /v1/models failed: {err}"),
        }
    }

    println!("stopping …");
    manager.stop(&instance.id).await.expect("stop");
    println!(
        "stopped: {:?}",
        manager.view(&instance.id).map(|v| v.status)
    );

    manager.stop_all().await;
}
