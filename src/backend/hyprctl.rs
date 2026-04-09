use hyprland::event_listener::EventListener;
use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WindowData {
    pub address: String,
    pub title: String,
    pub class: String,
}

#[derive(Clone)]
pub struct ActiveState {
    pub address: String,
    pub monitor: String,
}

#[derive(Deserialize)]
struct HyprctlClient {
    address: String,
    title: String,
    class: String,
    mapped: bool,
}

#[derive(Deserialize)]
struct HyprctlActiveWindow {
    address: String,
    monitor: i32,
}

#[derive(Deserialize)]
struct HyprctlActiveWindowAlt {
    address: String,
    #[serde(rename = "monitorID")]
    monitor: i32,
}

#[derive(Deserialize)]
struct HyprctlMonitor {
    id: i32,
    name: String,
}

/// Spawns the main Hyprland IPC bridge orchestrating both the application window list
/// and the real-time background active-workspace snapshot mechanism.
pub fn spawn_listener(sender: async_channel::Sender<Vec<WindowData>>) {
    thread::spawn(move || {
        let (signal_tx, signal_rx) = async_channel::unbounded::<()>();
        let signal_tx_clone = signal_tx.clone();

        thread::spawn(move || {
            let mut listener = EventListener::new();
            listener.add_active_window_change_handler(move |_| {
                let _ = signal_tx_clone.send_blocking(());
            });
            let _ = listener.start_listener();
        });

        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                let mut last_state: Option<ActiveState> = None;

                let _ = refresh_and_send(&sender, &mut last_state).await;

                if let Some(prev) = last_state.clone() {
                    let out_path = format!("{}/{}.png", crate::config::THUMBNAIL_DIR, prev.address);
                    let _ = crate::backend::screencopy::capture_active_workspace(
                        &out_path,
                        &prev.monitor,
                    )
                    .await;
                }

                loop {
                    tokio::select! {
                        recv_result = signal_rx.recv() => {
                            if recv_result.is_err() {
                                break;
                            }

                            if let Some(prev) = last_state.clone() {
                                let out_path = format!("{}/{}.png", crate::config::THUMBNAIL_DIR, prev.address);
                                let _ = crate::backend::screencopy::capture_active_workspace(
                                    &out_path,
                                    &prev.monitor,
                                )
                                .await;
                            }

                            let _ = refresh_and_send(&sender, &mut last_state).await;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(250)) => {
                            let _ = refresh_and_send(&sender, &mut last_state).await;
                        }
                    }
                }
            });
        }
    });
}

/// Pushes the latest valid window array out to the UI layer rendering stream
/// and subsequently spawns fire-and-forget Garbage Collection.
async fn refresh_and_send(
    sender: &async_channel::Sender<Vec<WindowData>>,
    last_state: &mut Option<ActiveState>,
) -> Result<(), ()> {
    if let Some(clients) = run_hyprctl_json::<Vec<HyprctlClient>>(&["clients", "-j"]) {
        let mut windows = Vec::new();
        let mut active_addresses = HashSet::new();

        for client in clients {
            if !client.title.is_empty() && client.mapped {
                active_addresses.insert(client.address.clone());
                windows.push(WindowData {
                    address: client.address,
                    title: client.title,
                    class: client.class,
                });
            }
        }

        let _ = sender.send(windows).await;
        gc_thumbnails(active_addresses).await;
    }

    if let Some(active_window) = fetch_active_window() {
        if active_window.address.is_empty() {
            return Ok(());
        }

        if let Some(monitors) = run_hyprctl_json::<Vec<HyprctlMonitor>>(&["monitors", "-j"]) {
            if let Some(active_monitor_name) = monitors
                .into_iter()
                .find(|monitor| monitor.id == active_window.monitor)
                .map(|monitor| monitor.name)
            {
                *last_state = Some(ActiveState {
                    address: active_window.address,
                    monitor: active_monitor_name,
                });
            }
        }
    }

    Ok(())
}

fn fetch_active_window() -> Option<HyprctlActiveWindow> {
    run_hyprctl_json::<HyprctlActiveWindow>(&["activewindow", "-j"]).or_else(|| {
        run_hyprctl_json::<HyprctlActiveWindowAlt>(&["activewindow", "-j"]).map(|active| {
            HyprctlActiveWindow {
                address: active.address,
                monitor: active.monitor,
            }
        })
    })
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

/// Iterates asynchronously over the RAM tmpfs disk validating that only perfectly active
/// windows retain cached snapshot thumbnails. Orphan files are discarded transparently.
async fn gc_thumbnails(active_addresses: HashSet<String>) {
    tokio::spawn(async move {
        if let Ok(mut entries) = tokio::fs::read_dir(crate::config::THUMBNAIL_DIR).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        let file_name = entry.file_name();
                        let name_str = file_name.to_string_lossy();
                        if name_str.ends_with(".png") {
                            let prefix = name_str.trim_end_matches(".png");
                            if !active_addresses.contains(prefix) {
                                let _ = tokio::fs::remove_file(entry.path()).await;
                            }
                        }
                    }
                }
            }
        }
    });
}
