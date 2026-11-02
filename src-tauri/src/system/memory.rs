//! Machine memory introspection via `sysinfo`.

use std::collections::HashSet;

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
///
/// The MLX engine may run as a grandchild of the `mtplx serve` process, so
/// attributing memory to the direct child alone would badly under-report.
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

/// Sum resident memory of every process that advertises `port` on its command
/// line.
///
/// A server process that gets re-parented to `launchd` is no longer reachable
/// through the process tree, but its command line still carries the port.
pub fn processes_serving_port(sys: &mut System, port: u16) -> u64 {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );

    let needle = format!(":{port}");
    let port_flag = format!("--port={port}");
    let mut total = 0u64;

    for process in sys.processes().values() {
        let cmd = process.cmd();
        let joined: Vec<String> = cmd
            .iter()
            .map(|part| part.to_string_lossy().to_string())
            .collect();
        let line = joined.join(" ");
        if line.contains(&needle) || line.contains(&port_flag) {
            total = total.saturating_add(process.memory());
        }
    }

    total
}
