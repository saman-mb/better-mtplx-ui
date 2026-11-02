//! Machine memory introspection via `sysinfo`.
//!
//! Two attribution strategies are needed on macOS:
//!
//! * **Process tree** — for servers this app started. `mtplx serve` execs into
//!   the engine process, so the child we spawned *is* the server and the tree
//!   walk gives the exact footprint.
//! * **Listening socket** — for servers this app did not start. `sysinfo` cannot
//!   read the command line of an interpreter whose path contains spaces (the
//!   MTPLX runtime venv), so matching on argv is unreliable. `lsof` reports the
//!   owning pid directly, which always works.

use std::collections::HashSet;
use std::process::Command;

use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use super::super::instances::types::MemorySnapshot;

/// Build a `sysinfo` handle with the refresh kinds this app needs.
pub fn new_system() -> System {
    let mut sys = System::new();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory().with_cpu(),
    );
    sys
}

/// Current machine memory usage.
pub fn snapshot(sys: &mut System) -> MemorySnapshot {
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());
    let total = sys.total_memory();
    let available = sys.available_memory();
    let used = total.saturating_sub(available);
    MemorySnapshot {
        total_bytes: total,
        available_bytes: available,
        used_bytes: used,
        used_ratio: if total == 0 {
            0.0
        } else {
            used as f64 / total as f64
        },
    }
}

/// Sum resident memory across `pid` and all of its descendants.
pub fn process_tree_memory(sys: &mut System, pid: u32) -> u64 {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );

    let mut total = 0u64;
    let mut frontier = vec![Pid::from_u32(pid)];
    let mut visited: HashSet<Pid> = HashSet::new();

    while let Some(current) = frontier.pop() {
        if !visited.insert(current) {
            continue;
        }
        let Some(process) = sys.process(current) else {
            continue;
        };
        total = total.saturating_add(process.memory());
        for (child_pid, child) in sys.processes() {
            if child.parent() == Some(current) {
                frontier.push(*child_pid);
            }
        }
    }

    total
}

/// Pids listening on a loopback TCP port.
///
/// Uses `lsof`, which reports socket ownership reliably regardless of how the
/// owning process was launched.
pub fn pids_listening_on(port: u16) -> Vec<u32> {
    let output = Command::new("/usr/sbin/lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect()
}

/// Resident memory held by whatever is listening on `port`.
pub fn memory_for_port(sys: &mut System, port: u16) -> u64 {
    pids_listening_on(port)
        .into_iter()
        .map(|pid| process_tree_memory(sys, pid))
        .max()
        .unwrap_or(0)
}
