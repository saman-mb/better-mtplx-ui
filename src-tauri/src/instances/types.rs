//! Public shape of a managed model instance.

use serde::{Deserialize, Serialize};

/// Lifecycle of one `mtplx serve` process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// Process spawned, engine has not reported readiness yet.
    Starting,
    /// Engine is loading weights into unified memory.
    Loading,
    /// HTTP endpoint answered a probe.
    Ready,
    /// Stop requested, waiting for the process to exit.
    Stopping,
    /// Process exited with an error.
    Failed,
    /// Process exited cleanly (user-initiated stop).
    Stopped,
}

impl InstanceStatus {
    /// True while the instance still owns a port / memory footprint.
    pub fn is_live(self) -> bool {
        matches!(
            self,
            InstanceStatus::Starting
                | InstanceStatus::Loading
                | InstanceStatus::Ready
                | InstanceStatus::Stopping
        )
    }
}

/// A single running (or recently stopped) model instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceView {
    /// Stable identifier, also used as the Tauri event payload key.
    pub id: String,
    /// Hugging Face repo id of the loaded pack.
    pub model: String,
    /// Port the instance is bound to.
    pub port: u16,
    /// Current lifecycle state.
    pub status: InstanceStatus,
    /// Resident memory of the instance's process tree, in bytes.
    pub memory_bytes: u64,
    /// Weight footprint the engine reports it mapped into unified memory.
    ///
    /// This is the number that decides whether the model fits alongside the
    /// others, and it is larger than the resident set for memory-mapped
    /// weights that have not been paged in yet.
    pub weights_bytes: Option<u64>,
    /// Number of tokens per second reported by the engine, when known.
    pub tokens_per_second: Option<f64>,
    /// Last error message, if the instance failed.
    pub error: Option<String>,
    /// Recent log lines, newest last.
    pub logs: Vec<String>,
    /// Unix timestamp (seconds) the instance was spawned.
    pub started_at: Option<u64>,
}

/// Snapshot of machine memory, used for the memory gauge and pre-flight checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    /// Physical (unified) memory installed, in bytes.
    pub total_bytes: u64,
    /// Memory available for new allocations, in bytes.
    pub available_bytes: u64,
    /// Memory currently in use, in bytes.
    pub used_bytes: u64,
    /// Fraction of total memory in use, 0.0 – 1.0.
    pub used_ratio: f64,
}
