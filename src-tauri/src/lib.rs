//! Better MTPLX UI — a lightweight, parallel-first GUI for MTPLX.
//!
//! The Rust side owns every side effect: it resolves the `mtplx` binary,
//! spawns and supervises `mtplx serve` processes, and reports live state to
//! the frontend over a single `instances://tick` event.

pub mod instances;
pub mod mtplx;
pub mod settings;
pub mod system;

use std::sync::Arc;

use tauri::{Emitter, Manager, State};

use instances::manager::InstanceManager;
use instances::types::{InstanceView, MemorySnapshot};
use mtplx::cli;
use mtplx::types::{DiscoverCard, InspectResult, ModelsPayload, StatusPayload};

/// Event carrying the full instance + memory state, emitted once per second.
pub const TICK_EVENT: &str = "instances://tick";

/// Ports probed for MTPLX servers this app did not start.
const DISCOVERY_RANGE: std::ops::RangeInclusive<u16> = 8000..=8015;

/// How many ticks between discovery scans.
const DISCOVERY_EVERY_TICKS: u64 = 5;

/// Shared application state.
pub struct AppState {
    pub manager: Arc<InstanceManager>,
    pub settings: settings::SettingsStore,
}

/// Everything the frontend needs in one payload, so the UI renders from a
/// single event rather than polling several commands.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TickPayload {
    pub instances: Vec<InstanceView>,
    pub memory: MemorySnapshot,
    /// MTPLX servers running on this Mac that this app did not start.
    pub external: Vec<instances::ExternalServer>,
}

// ---------------------------------------------------------------------------
// Model library
// ---------------------------------------------------------------------------

/// List cached MTPLX packs.
#[tauri::command]
async fn list_models() -> Result<ModelsPayload, String> {
    cli::list_models().await.map_err(|e| e.to_string())
}

/// Start downloading a pack (non-blocking; poll `list_models` for progress).
#[tauri::command]
async fn pull_model(repo_id: String) -> Result<(), String> {
    cli::pull(&repo_id).await.map_err(|e| e.to_string())
}

/// Inspect a pack for runtime-contract and MTP compatibility.
#[tauri::command]
async fn inspect_model(repo_id: String, require_mtp: bool) -> Result<InspectResult, String> {
    cli::inspect(&repo_id, require_mtp)
        .await
        .map_err(|e| e.to_string())
}

/// Fetch install / model / integration health.
#[tauri::command]
async fn mtplx_status() -> Result<StatusPayload, String> {
    cli::status().await.map_err(|e| e.to_string())
}

/// Browse MTPLX-branded models published on Hugging Face.
#[tauri::command]
async fn discover_models(
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<DiscoverCard>, String> {
    cli::discover(query.as_deref(), limit.unwrap_or(30))
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Instances
// ---------------------------------------------------------------------------

/// Start a model instance on the given port.
#[tauri::command]
async fn start_instance(
    manager: State<'_, Arc<InstanceManager>>,
    model: String,
    port: u16,
    extra_args: Vec<String>,
) -> Result<InstanceView, String> {
    manager
        .start(&model, port, &extra_args)
        .await
        .map_err(|e| e.to_string())
}

/// Stop a model instance and release its port.
#[tauri::command]
async fn stop_instance(manager: State<'_, Arc<InstanceManager>>, id: String) -> Result<(), String> {
    manager.stop(&id).await.map_err(|e| e.to_string())
}

/// Stop an MTPLX server this app did not start, by port.
#[tauri::command]
async fn stop_external_server(port: u16) -> Result<(), String> {
    let port_arg = port.to_string();
    cli::run(&["stop", "--port", &port_arg])
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Scan the loopback interface for MTPLX servers this app did not start.
#[tauri::command]
async fn discover_servers() -> Result<Vec<instances::ExternalServer>, String> {
    let ports: Vec<u16> = DISCOVERY_RANGE.collect();
    let packs = cached_pack_sizes().await;
    Ok(instances::discovery::scan(&ports, &packs).await)
}

/// Sizes of the cached packs, used to resolve an external server's footprint.
async fn cached_pack_sizes() -> Vec<instances::discovery::PackSize> {
    cli::list_models()
        .await
        .map(|payload| {
            payload
                .models
                .iter()
                .map(|model| instances::discovery::PackSize {
                    repo_id: model.repo_id.clone(),
                    size_bytes: model.size_bytes.unwrap_or_else(|| {
                        (model.size_gb.unwrap_or(0.0) * 1024.0 * 1024.0 * 1024.0) as u64
                    }),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Current view of every instance.
#[tauri::command]
async fn list_instances(
    manager: State<'_, Arc<InstanceManager>>,
) -> Result<Vec<InstanceView>, String> {
    Ok(manager.views())
}

/// Machine memory snapshot.
#[tauri::command]
async fn memory_snapshot(
    manager: State<'_, Arc<InstanceManager>>,
) -> Result<MemorySnapshot, String> {
    Ok(manager.memory_snapshot())
}

/// Suggest a free TCP port for a new instance.
#[tauri::command]
async fn suggest_port(manager: State<'_, Arc<InstanceManager>>) -> Result<u16, String> {
    let used = manager.used_ports();
    for candidate in 8000u16..8100 {
        if !used.contains(&candidate) {
            return Ok(candidate);
        }
    }
    Err("no free port in 8000-8099".to_string())
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Read the persisted settings.
#[tauri::command]
async fn get_settings(
    store: State<'_, settings::SettingsStore>,
) -> Result<settings::Settings, String> {
    Ok(store.get())
}

/// Persist the settings.
#[tauri::command]
async fn set_settings(
    store: State<'_, settings::SettingsStore>,
    settings: settings::Settings,
) -> Result<settings::Settings, String> {
    store.set(settings).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Build and run the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = settings::SettingsStore::new(app.handle()).expect("load settings");
            let manager = Arc::new(InstanceManager::new());

            app.manage(AppState {
                manager: Arc::clone(&manager),
                settings: store,
            });

            // Single ticker drives both supervision and UI updates.
            let tick_handle = app.handle().clone();
            let tick_manager = Arc::clone(&manager);
            tauri::async_runtime::spawn(async move {
                let mut tick_count: u64 = 0;
                let mut external: Vec<instances::ExternalServer> = Vec::new();
                loop {
                    tick_manager.refresh().await;
                    tick_count += 1;

                    // Scanning the port range is the only outbound work, so it
                    // runs less often than the local supervision pass.
                    if tick_count % DISCOVERY_EVERY_TICKS == 1 {
                        let ports: Vec<u16> = DISCOVERY_RANGE.collect();
                        let packs = cached_pack_sizes().await;
                        external = instances::discovery::scan(&ports, &packs).await;
                    }

                    let payload = TickPayload {
                        instances: tick_manager.views(),
                        memory: tick_manager.memory_snapshot(),
                        external: external.clone(),
                    };
                    let _ = tick_handle.emit(TICK_EVENT, payload);
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Leaving the app should not leave orphaned model servers running.
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(state) = window.app_handle().try_state::<AppState>() {
                    let manager = Arc::clone(&state.manager);
                    tauri::async_runtime::spawn(async move {
                        manager.stop_all().await;
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_models,
            pull_model,
            inspect_model,
            mtplx_status,
            discover_models,
            start_instance,
            stop_instance,
            stop_external_server,
            discover_servers,
            list_instances,
            memory_snapshot,
            suggest_port,
            get_settings,
            set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Better MTPLX UI");
}
