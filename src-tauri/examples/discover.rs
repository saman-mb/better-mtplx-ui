//! Print the MTPLX servers discovered on the loopback interface.
//!
//! Run with: `cargo run --example discover`

#[tokio::main]
async fn main() {
    let ports: Vec<u16> = (8000..=8015).collect();
    let found = better_mtplx_ui_lib::instances::discovery::scan(&ports, &[]).await;

    if found.is_empty() {
        println!("no MTPLX servers found on ports {ports:?}");
        return;
    }

    for server in found {
        println!(
            "port={} model={} context={:?}",
            server.port, server.model_id, server.context_length
        );
    }
}
