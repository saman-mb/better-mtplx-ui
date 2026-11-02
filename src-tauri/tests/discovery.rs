//! Tests for discovering MTPLX servers this app did not start.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use better_mtplx_ui_lib::instances::discovery;

/// Spawn a tiny HTTP server that answers `GET /v1/models` with `body`.
///
/// Returns the port it is listening on.
fn spawn_stub_http(port: u16, body: &'static str) -> std::thread::JoinHandle<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind stub");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            // Read the request line and headers, then answer.
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    })
}

#[tokio::test]
async fn discovery_finds_a_running_models_endpoint() {
    let body = r#"{"object":"list","data":[{"id":"stub-model-x","object":"model","max_context_length":262144}]}"#;
    let handle = spawn_stub_http(18190, body);

    // Give the listener a moment to accept.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let found = discovery::scan(&[18190, 18191], &[]).await;

    assert_eq!(
        found.len(),
        1,
        "exactly one endpoint should answer: {found:?}"
    );
    assert_eq!(found[0].port, 18190);
    assert_eq!(found[0].model_id, "stub-model-x");
    assert_eq!(found[0].context_length, Some(262144));

    // The closed port must not appear.
    assert!(!found.iter().any(|server| server.port == 18191));

    drop(handle);
}

#[tokio::test]
async fn discovery_ignores_ports_that_do_not_speak_the_api() {
    // A plain TCP listener that never answers with JSON must be ignored.
    let listener = TcpListener::bind(("127.0.0.1", 18192)).expect("bind");
    std::thread::spawn(move || {
        let mut held = Vec::new();
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                held.push(stream);
            }
        }
    });

    let found = discovery::scan(&[18192], &[]).await;
    assert!(
        found.is_empty(),
        "non-API port must not be reported: {found:?}"
    );
}
