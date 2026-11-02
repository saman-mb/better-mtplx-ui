//! Spawning, supervising and stopping parallel `mtplx serve` processes.

use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::mtplx::cli;
use crate::system::memory;

use super::types::{InstanceStatus, InstanceView};

/// How many log lines we keep per instance.
const LOG_CAPACITY: usize = 400;

/// A managed instance: the record plus the OS handle we need to supervise it.
struct ManagedInstance {
    id: String,
    model: String,
    port: u16,
    status: InstanceStatus,
    pid: Option<u32>,
    child: Option<Child>,
    error: Option<String>,
    logs: VecDeque<String>,
    started_at: Option<u64>,
    tokens_per_second: Option<f64>,
}

impl ManagedInstance {
    fn to_view(&self, memory_bytes: u64) -> InstanceView {
        InstanceView {
            id: self.id.clone(),
            model: self.model.clone(),
            port: self.port,
            status: self.status,
            memory_bytes,
            tokens_per_second: self.tokens_per_second,
            error: self.error.clone(),
            logs: self.logs.iter().cloned().collect(),
            started_at: self.started_at,
        }
    }
}

/// Owns every running instance plus the machine's process/memory tables.
pub struct InstanceManager {
    instances: Mutex<HashMap<String, ManagedInstance>>,
    system: Mutex<sysinfo::System>,
    counter: Mutex<u64>,
}

impl InstanceManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
            system: Mutex::new(memory::new_system()),
            counter: Mutex::new(0),
        }
    }

    /// Snapshot of machine memory.
    pub fn memory_snapshot(&self) -> crate::instances::types::MemorySnapshot {
        let mut sys = self.system.lock();
        memory::snapshot(&mut sys)
    }

    /// Current view of every instance.
    pub fn views(&self) -> Vec<InstanceView> {
        let mut sys = self.system.lock();
        let instances = self.instances.lock();
        let mut views: Vec<InstanceView> = instances
            .values()
            .map(|instance| {
                // The engine may be a descendant (or a re-parented orphan), so
                // take the larger of the tree total and the port-matched total.
                let tree = instance
                    .pid
                    .map(|pid| memory::process_tree_memory(&mut sys, pid))
                    .unwrap_or(0);
                let by_port = memory::processes_serving_port(&mut sys, instance.port);
                instance.to_view(tree.max(by_port))
            })
            .collect();
        views.sort_by_key(|view| view.started_at.unwrap_or(0));
        views
    }

    /// Look up a single instance view.
    pub fn view(&self, id: &str) -> Option<InstanceView> {
        self.views().into_iter().find(|view| view.id == id)
    }

    /// Ports currently held by a live instance.
    pub fn used_ports(&self) -> Vec<u16> {
        self.instances
            .lock()
            .values()
            .filter(|instance| instance.status.is_live())
            .map(|instance| instance.port)
            .collect()
    }

    /// Start a new instance for `model` on `port`.
    ///
    /// `extra_args` are passed through to `mtplx serve` verbatim, letting the
    /// user override profile, context window, scheduler mode and so on.
    pub async fn start(
        self: &Arc<Self>,
        model: &str,
        port: u16,
        extra_args: &[String],
    ) -> Result<InstanceView> {
        if port == 0 {
            bail!("port must be a non-zero TCP port")
        }
        if self.used_ports().contains(&port) {
            bail!("port {port} is already in use by another instance")
        }

        let id = {
            let mut counter = self.counter.lock();
            *counter += 1;
            format!("inst-{port}-{}", *counter)
        };

        let mut args: Vec<String> = vec![
            "serve".into(),
            "--model".into(),
            model.into(),
            "--port".into(),
            port.to_string(),
            "--host".into(),
            "127.0.0.1".into(),
            // Never let the engine block on an interactive confirmation prompt.
            "--yes".into(),
        ];
        args.extend(extra_args.iter().cloned());

        let bin = cli::resolve_binary()?;
        let mut child = Command::new(&bin)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false)
            .spawn()
            .with_context(|| format!("failed to spawn `mtplx serve` for {model}"))?;
        let pid = child.id();

        // Drain stdout/stderr into the per-instance log buffer.
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        if let Some(stdout) = child.stdout.take() {
            spawn_log_reader(stdout, tx.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(stderr, tx);
        }

        let manager = Arc::clone(self);
        let reader_id = id.clone();
        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                manager.push_log(&reader_id, line);
            }
        });

        let instance = ManagedInstance {
            id: id.clone(),
            model: model.to_string(),
            port,
            status: InstanceStatus::Starting,
            pid,
            child: Some(child),
            error: None,
            logs: VecDeque::new(),
            started_at: Some(unix_now()),
            tokens_per_second: None,
        };
        self.instances.lock().insert(id.clone(), instance);

        Ok(self
            .view(&id)
            .context("instance vanished right after start")?)
    }

    /// Stop an instance and release its port.
    pub async fn stop(self: &Arc<Self>, id: &str) -> Result<()> {
        let (port, mut child) = {
            let mut instances = self.instances.lock();
            let instance = instances.get_mut(id).context("no such instance")?;
            instance.status = InstanceStatus::Stopping;
            (instance.port, instance.child.take())
        };

        // The sanctioned shutdown path: let the engine release the port.
        let port_arg = port.to_string();
        let _ = cli::run(&["stop", "--port", &port_arg]).await;

        if let Some(child) = child.as_mut() {
            if let Some(pid) = child.id() {
                send_sigterm(pid as i32);
            }
            let graceful = tokio::time::timeout(Duration::from_secs(8), child.wait()).await;
            if graceful.is_err() {
                let _ = child.start_kill();
            }
        }

        if let Some(instance) = self.instances.lock().get_mut(id) {
            instance.status = InstanceStatus::Stopped;
            instance.pid = None;
        }
        Ok(())
    }

    /// Stop every live instance (used on app shutdown).
    pub async fn stop_all(self: &Arc<Self>) {
        let ids: Vec<String> = self
            .instances
            .lock()
            .values()
            .filter(|instance| instance.status.is_live())
            .map(|instance| instance.id.clone())
            .collect();
        for id in ids {
            let _ = self.stop(&id).await;
        }
    }

    /// Append a log line to an instance's buffer.
    fn push_log(&self, id: &str, line: String) {
        let mut instances = self.instances.lock();
        let Some(instance) = instances.get_mut(id) else {
            return;
        };
        // Track a throughput figure if the engine reports one.
        if let Some(tps) = parse_tokens_per_second(&line) {
            instance.tokens_per_second = Some(tps);
        }
        if instance.logs.len() >= LOG_CAPACITY {
            instance.logs.pop_front();
        }
        instance.logs.push_back(line);
    }

    /// Refresh liveness and readiness for every instance.
    pub async fn refresh(&self) {
        // Reap any process that exited on its own.
        {
            let mut instances = self.instances.lock();
            for instance in instances.values_mut() {
                let Some(child) = instance.child.as_mut() else {
                    continue;
                };
                match child.try_wait() {
                    Ok(Some(exit)) => {
                        instance.child = None;
                        instance.pid = None;
                        if exit.success() {
                            instance.status = InstanceStatus::Stopped;
                        } else {
                            instance.status = InstanceStatus::Failed;
                            instance.error = Some(format!("mtplx serve exited with {exit}"));
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        instance.child = None;
                        instance.pid = None;
                        instance.status = InstanceStatus::Failed;
                        instance.error = Some(format!("failed to poll mtplx serve: {err}"));
                    }
                }
            }
        }

        // Probe readiness for instances that are not up yet.
        let candidates: Vec<(String, u16)> = self
            .instances
            .lock()
            .values()
            .filter(|instance| {
                instance.status == InstanceStatus::Starting
                    || instance.status == InstanceStatus::Loading
            })
            .map(|instance| (instance.id.clone(), instance.port))
            .collect();

        for (id, port) in candidates {
            if port_is_open(port).await {
                if let Some(instance) = self.instances.lock().get_mut(&id) {
                    instance.status = InstanceStatus::Ready;
                }
            } else if let Some(instance) = self.instances.lock().get_mut(&id) {
                if instance.status == InstanceStatus::Starting {
                    instance.status = InstanceStatus::Loading;
                }
            }
        }
    }
}

/// Spawn a task that forwards each line of `reader` to `tx`.
fn spawn_log_reader<R>(reader: R, tx: mpsc::UnboundedSender<String>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
}

/// True when something is listening on `port` on loopback.
async fn port_is_open(port: u16) -> bool {
    let connect = TcpStream::connect(("127.0.0.1", port));
    tokio::time::timeout(Duration::from_millis(400), connect)
        .await
        .map(|result| result.is_ok())
        .unwrap_or(false)
}

/// Pull a token-per-second figure out of an engine log line.
fn parse_tokens_per_second(line: &str) -> Option<f64> {
    let lower = line.to_ascii_lowercase();
    let idx = lower.find("tok/s").or_else(|| lower.find("tps"))?;
    let digits: String = line[..idx]
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let value: f64 = digits.chars().rev().collect::<String>().parse().ok()?;
    if value > 0.0 { Some(value) } else { None }
}

/// Current Unix timestamp in seconds.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Send SIGTERM to a process id.
#[cfg(unix)]
fn send_sigterm(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
}

#[cfg(not(unix))]
fn send_sigterm(_pid: i32) {}
