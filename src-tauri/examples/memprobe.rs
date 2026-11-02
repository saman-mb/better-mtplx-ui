//! Print how memory attribution resolves for a running MTPLX server.
//!
//! Run with: `cargo run --example memprobe -- <port> [pid]`

use better_mtplx_ui_lib::system::memory;

fn main() {
    let mut args = std::env::args().skip(1);
    let port: u16 = args.next().and_then(|v| v.parse().ok()).unwrap_or(8000);
    let pid: Option<u32> = args.next().and_then(|v| v.parse().ok());

    let mut sys = memory::new_system();

    if let Some(pid) = pid {
        let tree = memory::process_tree_memory(&mut sys, pid);
        println!(
            "tree total for pid {pid}: {:.2} GB",
            tree as f64 / 1024f64.powi(3)
        );
    }

    let by_port = memory::memory_for_port(&mut sys, port);
    println!(
        "port total for {port}: {:.2} GB",
        by_port as f64 / 1024f64.powi(3)
    );
    println!(
        "pids listening on {port}: {:?}",
        memory::pids_listening_on(port)
    );

    println!("\n--- all processes holding > 1 GB ---");
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::nothing().with_memory(),
    );
    let mut rows: Vec<(u32, u32, u64, String)> = sys
        .processes()
        .values()
        .filter(|p| p.memory() > 1024 * 1024 * 1024)
        .map(|p| {
            let cmd = p
                .cmd()
                .iter()
                .map(|c| c.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            (
                p.pid().as_u32(),
                p.parent().map(|x| x.as_u32()).unwrap_or(0),
                p.memory(),
                cmd,
            )
        })
        .collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.2));
    for (pid, ppid, bytes, cmd) in rows {
        println!(
            "pid={pid} ppid={ppid} {:.2} GB | {}",
            bytes as f64 / 1024f64.powi(3),
            &cmd[..cmd.len().min(150)]
        );
    }
}
