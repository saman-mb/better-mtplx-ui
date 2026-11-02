//! Integration tests for the parallel-instance orchestrator.
//!
//! The tests drive a stub `mtplx` binary so they exercise the real spawn /
//! supervise / stop path without loading a real model.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use better_mtplx_ui_lib::instances::manager::InstanceManager;
use better_mtplx_ui_lib::instances::types::InstanceStatus;
use better_mtplx_ui_lib::settings::SettingsStore;

/// One shared stub binary for the whole test binary, so the `MTPLX_BIN`
/// override cannot race between concurrently running tests.
fn stub_binary() -> &'static PathBuf {
    static STUB: OnceLock<PathBuf> = OnceLock::new();
    STUB.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("bmtplx-stub-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create stub dir");
        let path = dir.join("mtplx");
        std::fs::write(
            &path,
            r#"#!/bin/bash
# Stub MTPLX. `serve` binds the requested port and idles until terminated.
if [ "$1" = "serve" ]; then
  PORT=0
  MODEL="unknown"
  while [ $# -gt 0 ]; do
    case "$1" in
      --port) PORT="$2"; shift 2 ;;
      --port=*) PORT="${1#*=}"; shift ;;
      --model) MODEL="$2"; shift 2 ;;
      --model=*) MODEL="${1#*=}"; shift ;;
      *) shift ;;
    esac
  done
  echo "stub serve model=$MODEL port=$PORT"
  exec python3 -c "
import socket, sys, time
port = int(sys.argv[1])
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('127.0.0.1', port))
s.listen(8)
print('stub listening on', port, flush=True)
while True:
    time.sleep(3600)
" "$PORT"
fi
if [ "$1" = "stop" ]; then
  exit 0
fi
exit 0
"#,
        )
        .expect("write stub");
        use_shell::set_mode(&path, 0o755);
        path
    })
}

/// Tiny helper so the test does not need the `unix_permissions` crate.
mod use_shell {
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    pub fn set_mode(path: &Path, mode: u32) {
        let mut perms = std::fs::metadata(path).expect("metadata").permissions();
        perms.set_mode(mode);
        std::fs::set_permissions(path, perms).expect("chmod");
    }
}

fn use_stub() {
    // SAFETY: every test in this binary points at the same stub path, so the
    // value written here is identical regardless of ordering.
    unsafe { std::env::set_var("MTPLX_BIN", stub_binary()) };
}

#[tokio::test]
async fn two_instances_run_in_parallel_on_separate_ports() {
    use_stub();

    let manager = Arc::new(InstanceManager::new());

    let first = manager
        .start("test/model-a", 18081, &[])
        .await
        .expect("start a");
    let second = manager
        .start("test/model-b", 18082, &[])
        .await
        .expect("start b");

    assert_eq!(first.port, 18081);
    assert_eq!(second.port, 18082);
    assert_ne!(first.id, second.id);

    // Both instances are tracked simultaneously.
    let views = manager.views();
    assert_eq!(views.len(), 2, "both instances must be tracked at once");

    // Refreshing flips both to Ready once their ports answer.
    for _ in 0..50 {
        manager.refresh().await;
        if manager
            .views()
            .iter()
            .all(|view| view.status == InstanceStatus::Ready)
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let views = manager.views();
    for view in &views {
        assert_eq!(
            view.status,
            InstanceStatus::Ready,
            "instance on {} never became ready",
            view.port
        );
    }

    // Both ports must accept connections — that is what "parallel" means.
    for port in [18081u16, 18082] {
        let reachable = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .expect("connect should not hang")
        .is_ok();
        assert!(reachable, "port {port} should be listening");
    }

    // Ports are remembered as in-use, so a third instance cannot steal one.
    let used = manager.used_ports();
    assert!(used.contains(&18081) && used.contains(&18082));

    let clash = manager.start("test/model-c", 18081, &[]).await;
    assert!(clash.is_err(), "starting on a busy port must fail");

    // Stopping one instance leaves the other running.
    manager.stop(&first.id).await.expect("stop a");
    let views = manager.views();
    let a = views
        .iter()
        .find(|v| v.id == first.id)
        .expect("a still tracked");
    let b = views
        .iter()
        .find(|v| v.id == second.id)
        .expect("b still tracked");
    assert_eq!(a.status, InstanceStatus::Stopped);
    assert_eq!(b.status, InstanceStatus::Ready, "b must survive stopping a");

    manager.stop_all().await;
}

#[tokio::test]
async fn instance_logs_capture_engine_output() {
    use_stub();

    let manager = Arc::new(InstanceManager::new());
    let instance = manager
        .start("test/model-a", 18083, &[])
        .await
        .expect("start");

    // Give the stub a moment to print its banner.
    for _ in 0..30 {
        if manager
            .view(&instance.id)
            .map(|view| !view.logs.is_empty())
            .unwrap_or(false)
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let view = manager.view(&instance.id).expect("view");
    assert!(
        !view.logs.is_empty(),
        "engine stdout should land in the instance log buffer"
    );
    assert!(
        view.logs.iter().any(|line| line.contains("stub serve")),
        "expected the stub banner in {:?}",
        view.logs
    );

    manager.stop_all().await;
}

#[tokio::test]
async fn memory_snapshot_reports_real_totals() {
    let manager = InstanceManager::new();
    let snapshot = manager.memory_snapshot();
    assert!(snapshot.total_bytes > 0, "total memory must be detected");
    assert!(snapshot.available_bytes <= snapshot.total_bytes);
    assert!((0.0..=1.0).contains(&snapshot.used_ratio));
}

#[test]
fn settings_round_trip_through_disk() {
    let dir = std::env::temp_dir().join(format!("bmtplx-settings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let store = SettingsStore::from_dir(&dir);
    let defaults = store.get();
    assert_eq!(defaults.default_port, 8000);
    assert_eq!(defaults.theme, "dark");

    let mut changed = defaults.clone();
    changed.default_port = 8123;
    changed.extra_args = vec!["--reasoning-effort".into(), "high".into()];
    store.set(changed).expect("save settings");

    // A fresh store must read the persisted values back.
    let reopened = SettingsStore::from_dir(&dir);
    let loaded = reopened.get();
    assert_eq!(loaded.default_port, 8123);
    assert_eq!(loaded.extra_args, vec!["--reasoning-effort", "high"]);

    let _ = std::fs::remove_dir_all(&dir);
}
