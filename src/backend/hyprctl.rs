use hyprland::event_listener::EventListener;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{
    FOCUS_LOG_PATH,
    FOCUS_QUEUE_CAPACITY,
    MONITOR_REFRESH_SECS,
    REFRESH_DEBOUNCE_MS,
    SAFETY_POLL_SECS,
    THUMBNAIL_DIR,
    THUMBNAIL_QUEUE_CAPACITY,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThumbnailState {
    Pending,
    Ready,
    Missing,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiStatus {
    Loading,
    Ready,
    Empty,
    BackendDegraded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowData {
    pub address: String,
    pub title: String,
    pub class: String,
    pub is_active: bool,
    pub monitor_name: Option<String>,
    pub workspace_id: Option<i32>,
    pub last_focus_seq: u64,
    pub thumbnail_state: ThumbnailState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiSnapshot {
    pub items: Vec<WindowData>,
    pub selected_address: Option<String>,
    pub status: UiStatus,
    pub revision: u64,
}

#[derive(Clone, Debug)]
pub struct FocusCommand {
    pub address: String,
}

#[derive(Clone, Debug)]
pub struct ThumbnailJob {
    pub address: String,
    pub monitor_name: String,
}

#[derive(Clone)]
pub struct FocusController {
    sender: async_channel::Sender<FocusCommand>,
    pending: Arc<Mutex<HashSet<String>>>,
}

impl FocusController {
    pub fn enqueue(&self, address: &str) {
        if address.is_empty() {
            return;
        }

        let mut pending = match self.pending.lock() {
            Ok(pending) => pending,
            Err(_) => return,
        };

        if !pending.insert(address.to_string()) {
            return;
        }

        if self
            .sender
            .try_send(FocusCommand {
                address: address.to_string(),
            })
            .is_err()
        {
            pending.remove(address);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SnapshotContent {
    items: Vec<WindowData>,
    selected_address: Option<String>,
    status: UiStatus,
}

#[derive(Clone, Debug)]
struct ActiveState {
    address: String,
    monitor_name: String,
}

#[derive(Clone, Debug)]
struct ThumbnailResult {
    address: String,
    monitor_name: String,
    success: bool,
}

struct BackendState {
    revision: u64,
    focus_seq: u64,
    last_emitted: Option<SnapshotContent>,
    last_successful_items: Vec<WindowData>,
    last_status: UiStatus,
    last_active_state: Option<ActiveState>,
    last_focus_seq_by_address: HashMap<String, u64>,
    mru_order: Vec<String>,
    monitor_map: HashMap<i32, String>,
    last_monitor_refresh: Option<Instant>,
    gc_in_flight: Arc<AtomicBool>,
    last_gc_addresses: HashSet<String>,
    thumbnail_states: HashMap<String, ThumbnailState>,
    in_flight_thumbnail_jobs: HashSet<(String, String)>,
}

impl BackendState {
    fn new() -> Self {
        Self {
            revision: 0,
            focus_seq: 0,
            last_emitted: None,
            last_successful_items: Vec::new(),
            last_status: UiStatus::Loading,
            last_active_state: None,
            last_focus_seq_by_address: HashMap::new(),
            mru_order: Vec::new(),
            monitor_map: HashMap::new(),
            last_monitor_refresh: None,
            gc_in_flight: Arc::new(AtomicBool::new(false)),
            last_gc_addresses: HashSet::new(),
            thumbnail_states: HashMap::new(),
            in_flight_thumbnail_jobs: HashSet::new(),
        }
    }

    fn register_active_window(&mut self, address: &str) {
        if address.is_empty() {
            return;
        }

        if self
            .last_active_state
            .as_ref()
            .map(|state| state.address.as_str())
            == Some(address)
        {
            return;
        }

        self.focus_seq += 1;
        self.last_focus_seq_by_address
            .insert(address.to_string(), self.focus_seq);

        if let Some(existing_index) = self.mru_order.iter().position(|item| item == address) {
            self.mru_order.remove(existing_index);
        }
        self.mru_order.insert(0, address.to_string());
    }

    fn handle_active_transition(
        &mut self,
        address: String,
        monitor_name: String,
    ) -> Option<ThumbnailJob> {
        if address.is_empty() {
            return None;
        }

        let previous = self.last_active_state.clone();
        let changed = previous
            .as_ref()
            .map(|state| state.address != address)
            .unwrap_or(true);

        if changed {
            self.register_active_window(&address);
        }

        self.last_active_state = Some(ActiveState {
            address: address.clone(),
            monitor_name: monitor_name.clone(),
        });

        if changed {
            return previous.map(|state| ThumbnailJob {
                address: state.address,
                monitor_name: state.monitor_name,
            });
        }

        None
    }

    fn should_refresh_monitors(&self) -> bool {
        self.monitor_map.is_empty()
            || self
                .last_monitor_refresh
                .map(|instant| instant.elapsed() >= Duration::from_secs(MONITOR_REFRESH_SECS))
                .unwrap_or(true)
    }

    fn current_selected_address(&self, items: &[WindowData]) -> Option<String> {
        if items.len() >= 2 {
            return Some(items[1].address.clone());
        }

        items.first().map(|item| item.address.clone())
    }

    fn resolve_thumbnail_state(&self, address: &str) -> ThumbnailState {
        if self
            .in_flight_thumbnail_jobs
            .iter()
            .any(|(candidate, _)| candidate == address)
        {
            return ThumbnailState::Pending;
        }

        if self.thumbnail_states.get(address) == Some(&ThumbnailState::Failed) {
            return ThumbnailState::Failed;
        }

        if Path::new(&thumbnail_path(address)).exists() {
            return ThumbnailState::Ready;
        }

        self.thumbnail_states
            .get(address)
            .cloned()
            .unwrap_or(ThumbnailState::Missing)
    }

    fn retain_active_metadata(&mut self, active_addresses: &HashSet<String>) {
        self.mru_order
            .retain(|address| active_addresses.contains(address));
        self.last_focus_seq_by_address
            .retain(|address, _| active_addresses.contains(address));
        self.thumbnail_states
            .retain(|address, _| active_addresses.contains(address));
    }
}

#[derive(Deserialize)]
struct HyprctlWorkspace {
    id: i32,
}

#[derive(Deserialize)]
struct HyprctlClient {
    address: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    class: String,
    mapped: bool,
    #[serde(default)]
    monitor: Option<i32>,
    #[serde(default)]
    workspace: Option<HyprctlWorkspace>,
}

#[derive(Deserialize)]
struct HyprctlActiveWindow {
    address: String,
    #[serde(alias = "monitorID")]
    monitor: i32,
}

#[derive(Deserialize)]
struct HyprctlMonitor {
    id: i32,
    name: String,
}

pub fn spawn_backend(sender: async_channel::Sender<UiSnapshot>) -> FocusController {
    let _ = sender.try_send(UiSnapshot {
        items: Vec::new(),
        selected_address: None,
        status: UiStatus::Loading,
        revision: 0,
    });

    let (focus_tx, focus_rx) = async_channel::bounded::<FocusCommand>(FOCUS_QUEUE_CAPACITY);
    let pending_focus = Arc::new(Mutex::new(HashSet::new()));
    spawn_focus_worker(focus_rx, pending_focus.clone());

    let (thumbnail_tx, thumbnail_rx) =
        async_channel::bounded::<ThumbnailJob>(THUMBNAIL_QUEUE_CAPACITY);
    let (thumbnail_result_tx, thumbnail_result_rx) =
        async_channel::unbounded::<ThumbnailResult>();
    spawn_thumbnail_worker(thumbnail_rx, thumbnail_result_tx);

    let backend_sender = sender.clone();
    thread::spawn(move || {
        let (signal_tx, signal_rx) = async_channel::unbounded::<()>();
        spawn_event_listener(signal_tx);

        if let Ok(runtime) = tokio::runtime::Runtime::new() {
            runtime.block_on(async move {
                let mut state = BackendState::new();
                let _ = refresh_monitors(&mut state);
                let _ = refresh_and_emit(&backend_sender, &thumbnail_tx, &mut state).await;

                let mut safety_poll =
                    tokio::time::interval(Duration::from_secs(SAFETY_POLL_SECS));
                safety_poll.tick().await;

                loop {
                    tokio::select! {
                        recv_result = signal_rx.recv() => {
                            if recv_result.is_err() {
                                break;
                            }

                            while let Ok(Ok(())) = tokio::time::timeout(
                                Duration::from_millis(REFRESH_DEBOUNCE_MS),
                                signal_rx.recv(),
                            )
                            .await
                            {}

                            let _ =
                                refresh_and_emit(&backend_sender, &thumbnail_tx, &mut state)
                                    .await;
                        }
                        recv_result = thumbnail_result_rx.recv() => {
                            let Ok(result) = recv_result else {
                                break;
                            };

                            if apply_thumbnail_result(&mut state, result) {
                                let cached_items = rebuild_cached_items(&state);
                                let status = if cached_items.is_empty() {
                                    UiStatus::Empty
                                } else {
                                    state.last_status.clone()
                                };
                                let _ = emit_snapshot(
                                    &backend_sender,
                                    &mut state,
                                    cached_items,
                                    status,
                                )
                                .await;
                            }
                        }
                        _ = safety_poll.tick() => {
                            let _ =
                                refresh_and_emit(&backend_sender, &thumbnail_tx, &mut state)
                                    .await;
                        }
                    }
                }
            });
        }
    });

    FocusController {
        sender: focus_tx,
        pending: pending_focus,
    }
}

fn spawn_event_listener(sender: async_channel::Sender<()>) {
    thread::spawn(move || {
        let mut listener = EventListener::new();
        listener.add_active_window_change_handler(move |_| {
            let _ = sender.send_blocking(());
        });
        let _ = listener.start_listener();
    });
}

fn spawn_focus_worker(
    receiver: async_channel::Receiver<FocusCommand>,
    pending: Arc<Mutex<HashSet<String>>>,
) {
    thread::spawn(move || {
        while let Ok(command) = receiver.recv_blocking() {
            let output = Command::new("hyprctl")
                .args(["dispatch", "focuswindow", &format!("address:{}", command.address)])
                .output();

            match output {
                Ok(output) if !output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                    log_focus_failure(
                        &command.address,
                        output.status.code(),
                        &stdout,
                        &stderr,
                    );
                }
                Err(error) => {
                    log_focus_failure(&command.address, None, "", &error.to_string());
                }
                Ok(_) => {}
            }

            if let Ok(mut pending) = pending.lock() {
                pending.remove(&command.address);
            }
        }
    });
}

fn spawn_thumbnail_worker(
    receiver: async_channel::Receiver<ThumbnailJob>,
    sender: async_channel::Sender<ThumbnailResult>,
) {
    thread::spawn(move || {
        let mut session = crate::backend::screencopy::ScreencopySession::connect().ok();

        while let Ok(job) = receiver.recv_blocking() {
            let out_path = thumbnail_path(&job.address);
            let mut success = false;

            if session.is_none() {
                session = crate::backend::screencopy::ScreencopySession::connect().ok();
            }

            if let Some(active_session) = session.as_mut() {
                success = active_session
                    .capture_active_workspace(&out_path, &job.monitor_name)
                    .is_ok();
            }

            if !success {
                session = crate::backend::screencopy::ScreencopySession::connect().ok();
                if let Some(active_session) = session.as_mut() {
                    success = active_session
                        .capture_active_workspace(&out_path, &job.monitor_name)
                        .is_ok();
                }
            }

            let _ = sender.send_blocking(ThumbnailResult {
                address: job.address,
                monitor_name: job.monitor_name,
                success,
            });
        }
    });
}

fn log_focus_failure(address: &str, exit_code: Option<i32>, stdout: &str, stderr: &str) {
    eprintln!(
        "Failed to focus {} (exit code: {:?})\nstdout: {}\nstderr: {}",
        address, exit_code, stdout, stderr
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(FOCUS_LOG_PATH)
    {
        let _ = writeln!(
            file,
            "[focus failure] address={} exit_code={:?}\nstdout={}\nstderr={}\n",
            address, exit_code, stdout, stderr
        );
    }
}

async fn refresh_and_emit(
    sender: &async_channel::Sender<UiSnapshot>,
    thumbnail_tx: &async_channel::Sender<ThumbnailJob>,
    state: &mut BackendState,
) -> Result<(), ()> {
    let clients = run_hyprctl_json::<Vec<HyprctlClient>>(&["clients", "-j"]);
    let active_window = fetch_active_window();
    let mut degraded = clients.is_none() || active_window.is_none();

    if state.should_refresh_monitors() && !refresh_monitors(state) {
        degraded = true;
    }

    let mut capture_job = None;
    if let Some(active_window) = active_window.as_ref() {
        let monitor_name =
            if let Some(monitor_name) = state.monitor_map.get(&active_window.monitor) {
                Some(monitor_name.clone())
            } else {
                if refresh_monitors(state) {
                    state.monitor_map.get(&active_window.monitor).cloned()
                } else {
                    degraded = true;
                    None
                }
            };

        if let Some(monitor_name) = monitor_name {
            capture_job =
                state.handle_active_transition(active_window.address.clone(), monitor_name);
        }
    }

    if let Some(job) = capture_job {
        enqueue_thumbnail_job(state, thumbnail_tx, job).await;
    }

    let items = if let Some(clients) = clients {
        let active_address = state
            .last_active_state
            .as_ref()
            .map(|state| state.address.as_str());
        let mut active_addresses = HashSet::new();
        let mut items = build_items(&clients, active_address, state, &mut active_addresses);

        maybe_run_gc(state, active_addresses).await;

        state.last_successful_items = items.clone();
        sort_items(&mut items);
        items
    } else {
        let mut items = rebuild_cached_items(state);
        sort_items(&mut items);
        items
    };

    let status = if degraded {
        UiStatus::BackendDegraded
    } else if items.is_empty() {
        UiStatus::Empty
    } else {
        UiStatus::Ready
    };

    emit_snapshot(sender, state, items, status).await
}

fn build_items(
    clients: &[HyprctlClient],
    active_address: Option<&str>,
    state: &mut BackendState,
    active_addresses: &mut HashSet<String>,
) -> Vec<WindowData> {
    let mut items = Vec::new();

    for client in clients {
        if !client.mapped {
            continue;
        }

        let is_active = active_address == Some(client.address.as_str());
        if client.title.is_empty() && !is_active {
            continue;
        }

        active_addresses.insert(client.address.clone());
        let fallback_class = if client.class.is_empty() {
            "application-x-executable".to_string()
        } else {
            client.class.clone()
        };
        let title = if client.title.is_empty() {
            fallback_class.clone()
        } else {
            client.title.clone()
        };

        items.push(WindowData {
            address: client.address.clone(),
            title,
            class: fallback_class,
            is_active,
            monitor_name: client
                .monitor
                .and_then(|monitor_id| state.monitor_map.get(&monitor_id).cloned()),
            workspace_id: client.workspace.as_ref().map(|workspace| workspace.id),
            last_focus_seq: state
                .last_focus_seq_by_address
                .get(&client.address)
                .copied()
                .unwrap_or(0),
            thumbnail_state: state.resolve_thumbnail_state(&client.address),
        });
    }

    state.retain_active_metadata(active_addresses);
    items
}

fn sort_items(items: &mut [WindowData]) {
    items.sort_by(|left, right| {
        right
            .last_focus_seq
            .cmp(&left.last_focus_seq)
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
    });
}

fn rebuild_cached_items(state: &BackendState) -> Vec<WindowData> {
    let mut items = state.last_successful_items.clone();
    for item in &mut items {
        item.last_focus_seq = state
            .last_focus_seq_by_address
            .get(&item.address)
            .copied()
            .unwrap_or(item.last_focus_seq);
        item.thumbnail_state = state.resolve_thumbnail_state(&item.address);
        item.is_active = state
            .last_active_state
            .as_ref()
            .map(|active| active.address == item.address)
            .unwrap_or(false);
    }
    items
}

async fn emit_snapshot(
    sender: &async_channel::Sender<UiSnapshot>,
    state: &mut BackendState,
    mut items: Vec<WindowData>,
    status: UiStatus,
) -> Result<(), ()> {
    sort_items(&mut items);
    let selected_address = state.current_selected_address(&items);
    let content = SnapshotContent {
        items: items.clone(),
        selected_address: selected_address.clone(),
        status: status.clone(),
    };

    if state.last_emitted.as_ref() == Some(&content) {
        return Ok(());
    }

    state.revision += 1;
    state.last_status = status.clone();
    state.last_emitted = Some(content);

    let _ = sender
        .send(UiSnapshot {
            items,
            selected_address,
            status,
            revision: state.revision,
        })
        .await;

    Ok(())
}

fn refresh_monitors(state: &mut BackendState) -> bool {
    let Some(monitors) = run_hyprctl_json::<Vec<HyprctlMonitor>>(&["monitors", "-j"]) else {
        return false;
    };

    state.monitor_map = monitors
        .into_iter()
        .map(|monitor| (monitor.id, monitor.name))
        .collect();
    state.last_monitor_refresh = Some(Instant::now());
    true
}

async fn maybe_run_gc(state: &mut BackendState, active_addresses: HashSet<String>) {
    if active_addresses == state.last_gc_addresses {
        return;
    }

    if state
        .gc_in_flight
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    state.last_gc_addresses = active_addresses.clone();
    let gc_in_flight = state.gc_in_flight.clone();
    tokio::spawn(async move {
        gc_thumbnails(active_addresses).await;
        gc_in_flight.store(false, Ordering::SeqCst);
    });
}

async fn enqueue_thumbnail_job(
    state: &mut BackendState,
    sender: &async_channel::Sender<ThumbnailJob>,
    job: ThumbnailJob,
) {
    let key = (job.address.clone(), job.monitor_name.clone());
    if !state.in_flight_thumbnail_jobs.insert(key) {
        return;
    }

    state
        .thumbnail_states
        .insert(job.address.clone(), ThumbnailState::Pending);

    if sender.send(job.clone()).await.is_err() {
        state
            .in_flight_thumbnail_jobs
            .remove(&(job.address.clone(), job.monitor_name.clone()));
        state
            .thumbnail_states
            .insert(job.address, ThumbnailState::Failed);
    }
}

fn apply_thumbnail_result(state: &mut BackendState, result: ThumbnailResult) -> bool {
    state
        .in_flight_thumbnail_jobs
        .remove(&(result.address.clone(), result.monitor_name));

    let next_state = if result.success {
        ThumbnailState::Ready
    } else {
        ThumbnailState::Failed
    };

    if state.thumbnail_states.get(&result.address) == Some(&next_state) {
        return false;
    }

    state.thumbnail_states.insert(result.address, next_state);
    true
}

fn fetch_active_window() -> Option<HyprctlActiveWindow> {
    run_hyprctl_json::<HyprctlActiveWindow>(&["activewindow", "-j"])
}

fn run_hyprctl_json<T>(args: &[&str]) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let output = Command::new("hyprctl").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

async fn gc_thumbnails(active_addresses: HashSet<String>) {
    if let Ok(mut entries) = tokio::fs::read_dir(THUMBNAIL_DIR).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if !file_type.is_file() {
                    continue;
                }

                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if !file_name.ends_with(".png") {
                    continue;
                }

                let address = file_name.trim_end_matches(".png");
                if !active_addresses.contains(address) {
                    let _ = tokio::fs::remove_file(entry.path()).await;
                }
            }
        }
    }
}

pub fn thumbnail_path(address: &str) -> String {
    format!("{}/{}.png", THUMBNAIL_DIR, address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_active_window_with_monitor_alias() {
        let payload = r#"{"address":"0x123","monitorID":2}"#;
        let parsed: HyprctlActiveWindow = serde_json::from_str(payload).unwrap();

        assert_eq!(parsed.address, "0x123");
        assert_eq!(parsed.monitor, 2);
    }

    #[test]
    fn selects_previous_window_when_multiple_items_exist() {
        let state = BackendState::new();
        let items = vec![
            WindowData {
                address: "0x1".into(),
                title: "Current".into(),
                class: "current".into(),
                is_active: true,
                monitor_name: None,
                workspace_id: None,
                last_focus_seq: 3,
                thumbnail_state: ThumbnailState::Ready,
            },
            WindowData {
                address: "0x2".into(),
                title: "Previous".into(),
                class: "previous".into(),
                is_active: false,
                monitor_name: None,
                workspace_id: None,
                last_focus_seq: 2,
                thumbnail_state: ThumbnailState::Ready,
            },
        ];

        assert_eq!(state.current_selected_address(&items), Some("0x2".into()));
    }

    #[test]
    fn sorts_items_by_latest_focus_sequence() {
        let mut items = vec![
            WindowData {
                address: "0x1".into(),
                title: "First".into(),
                class: "one".into(),
                is_active: false,
                monitor_name: None,
                workspace_id: None,
                last_focus_seq: 1,
                thumbnail_state: ThumbnailState::Missing,
            },
            WindowData {
                address: "0x2".into(),
                title: "Second".into(),
                class: "two".into(),
                is_active: true,
                monitor_name: None,
                workspace_id: None,
                last_focus_seq: 3,
                thumbnail_state: ThumbnailState::Missing,
            },
            WindowData {
                address: "0x3".into(),
                title: "Third".into(),
                class: "three".into(),
                is_active: false,
                monitor_name: None,
                workspace_id: None,
                last_focus_seq: 2,
                thumbnail_state: ThumbnailState::Missing,
            },
        ];

        sort_items(&mut items);

        assert_eq!(
            items
                .into_iter()
                .map(|item| item.address)
                .collect::<Vec<_>>(),
            vec!["0x2", "0x3", "0x1"]
        );
    }
}
