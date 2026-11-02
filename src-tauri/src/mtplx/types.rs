//! Serde types describing the JSON shapes emitted by the `mtplx` CLI.
//!
//! Every struct here mirrors a real `mtplx ... --json` payload. Fields that are
//! not always present are `Option` so a partial payload never fails to parse.

use serde::{Deserialize, Serialize};

/// A single model pack as reported by `mtplx models --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPack {
    pub repo_id: String,
    pub path: String,
    #[serde(default)]
    pub size_gb: Option<f64>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub has_runtime_contract: bool,
    #[serde(default)]
    pub recommended_profile: Option<String>,
    #[serde(default)]
    pub validation: Option<PackValidation>,
    #[serde(default)]
    pub delete_command: Option<String>,
}

/// Completeness report for a cached pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackValidation {
    #[serde(default)]
    pub contract_present: bool,
    #[serde(default)]
    pub missing_files: Vec<String>,
}

impl PackValidation {
    /// A pack is loadable only when its runtime contract is present and no
    /// files are missing. Anything else means "still downloading".
    pub fn is_complete(&self) -> bool {
        self.contract_present && self.missing_files.is_empty()
    }
}

/// Top-level shape of `mtplx models --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsPayload {
    #[serde(default)]
    pub cache_dir: String,
    #[serde(default)]
    pub models: Vec<ModelPack>,
}

/// Result of `mtplx inspect --json` for a single pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub compatible: Option<bool>,
    #[serde(default)]
    pub has_mtp: Option<bool>,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
}

/// Shape of `mtplx status --json`, narrowed to the fields we surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusPayload {
    #[serde(default)]
    pub install: Option<serde_json::Value>,
    #[serde(default)]
    pub server: Option<serde_json::Value>,
    #[serde(default)]
    pub diagnostics: Option<serde_json::Value>,
}

/// One model card from `mtplx forge discover --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverCard {
    /// Fully-qualified Hugging Face repo id, `owner/name`.
    pub repo: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub branded_name: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub license: Option<String>,
}
