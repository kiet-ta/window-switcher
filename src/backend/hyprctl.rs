use hyprland::data::{Clients, Client, Monitor};
use hyprland::shared::{Address, HyprData, HyprDataActiveOptional, HyprDataActive};
use hyprland::event_listener::EventListener;
use std::thread;
use std::collections::HashSet;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WindowData {
    pub address: Address,
    pub title: String,
    pub class: String,
}

#[derive(Clone)]
pub struct ActiveState {
    pub address: Address,
    pub monitor: String,
}

/// Spawns the main Hyprland IPC bridge orchestrating both the application window list
/// and the real-time background active-workspace snapshot mechanism.
pub fn spawn_listener(sender: async_channel::Sender<Vec<WindowData>>) {
    thread::spawn(move || {
        let (signal_tx, signal_rx) = async_channel::unbounded::<()>();
        let signal_tx_clone = signal_tx.clone();
        
        // Native synchronous listener running parallel to our Tokio async context.
        thread::spawn(move || {
            let mut listener = EventListener::new();
            listener.add_active_window_change_handler(move |_| {
                let _ = signal_tx_clone.send_blocking(());
            });
            let _ = listener.start_listener();
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut last_state: Option<ActiveState> = None;
            
            let _ = refresh_and_send(&sender, &mut last_state).await;
            
            while let Ok(_) = signal_rx.recv().await {
                if let Some(prev) = last_state.clone() {
                    let out_path = format!("{}/{}.png", crate::config::THUMBNAIL_DIR, prev.address);
                    let _ = crate::backend::screencopy::capture_active_workspace(&out_path, &prev.monitor).await;
                }
                let _ = refresh_and_send(&sender, &mut last_state).await;
            }
        });
    });
}

/// Pushes the latest valid window array out to the UI layer rendering stream
/// and subsequently spawns fire-and-forget Garbage Collection.
async fn refresh_and_send(
    sender: &async_channel::Sender<Vec<WindowData>>, 
    last_state: &mut Option<ActiveState>
) -> Result<(), ()> {
    if let Ok(clients) = Clients::get_async().await {
        let mut windows = Vec::new();
        let mut active_addresses = HashSet::new();
        
        for client in clients {
            if !client.title.is_empty() && client.mapped {
                active_addresses.insert(client.address.to_string());
                
                windows.push(WindowData {
                    address: client.address.clone(),
                    title: client.title.clone(),
                    class: client.class.clone(),
                });
            }
        }
        
        let _ = sender.send(windows).await;
        
        // Spawn Background Thumbnail GC
        gc_thumbnails(active_addresses).await;
    }
    
    if let Ok(Some(active)) = Client::get_active_async().await {
        if let Ok(monitor) = Monitor::get_active_async().await {
            *last_state = Some(ActiveState {
                address: active.address,
                monitor: monitor.name,
            });
        }
    }
    
    Ok(())
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
                                // Silent enforcement; errors ignored for non-blocking latency
                                let _ = tokio::fs::remove_file(entry.path()).await;
                            }
                        }
                    }
                }
            }
        }
    });
}
