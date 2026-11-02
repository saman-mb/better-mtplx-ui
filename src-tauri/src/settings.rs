//! Persisted user preferences.
//!
//! Settings live in the platform config directory (`~/Library/Application
//! Support/<app>/settings.json` on macOS) so they survive app updates.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// User-facing preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Port assigned to a new instance when the user does not pick one.
    pub default_port: u16,
    /// MTPLX profile used when starting an instance (`turbo`, `stable`, …).
    pub default_profile: String,
    /// Context window override in tokens; `None` leaves the profile default.
    pub context_window: Option<u32>,
    /// Interface the engine binds to. Loopback is the safe default.
    pub host: String,
    /// Optional API key for the local endpoints.
    pub api_key: Option<String>,
    /// Scheduler mode passed to `mtplx serve`.
    pub scheduler_mode: String,
    /// Extra raw arguments appended to every `mtplx serve` invocation.
    pub extra_args: Vec<String>,
    /// UI theme: `dark` or `light`.
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_port: 8000,
            default_profile: "turbo".to_string(),
            context_window: None,
            host: "127.0.0.1".to_string(),
            api_key: None,
            scheduler_mode: "serial".to_string(),
            extra_args: Vec::new(),
            theme: "dark".to_string(),
        }
    }
}

/// Loads and saves [`Settings`].
pub struct SettingsStore {
    path: PathBuf,
    current: parking_lot::Mutex<Settings>,
}

impl SettingsStore {
    /// Load settings from the platform config directory.
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("no config dir: {e}"))?;
        Ok(Self::from_dir(&dir))
    }

    /// Load settings from a specific directory.
    pub fn from_dir(dir: &std::path::Path) -> Self {
        let _ = std::fs::create_dir_all(dir);
        let path = dir.join("settings.json");

        let settings = std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .unwrap_or_default();

        Self {
            path,
            current: parking_lot::Mutex::new(settings),
        }
    }

    /// Current settings.
    pub fn get(&self) -> Settings {
        self.current.lock().clone()
    }

    /// Persist settings to disk.
    pub fn set(&self, settings: Settings) -> Result<Settings, String> {
        let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())?;
        *self.current.lock() = settings.clone();
        Ok(settings)
    }
}
