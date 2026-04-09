use hyprland::data::{Clients, Client};
use hyprland::shared::{Address, HyprData, HyprDataActiveOptional};
use hyprland::event_listener::EventListener;
use std::thread;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WindowData {
    pub address: Address,
    pub title: String,
    pub class: String,
}

pub fn spawn_listener(sender: async_channel::Sender<Vec<WindowData>>) {
    // Background Event Listener Hook (Task 4)
    thread::spawn(move || {
        // Channel to send signals from synchronous Hyprland callback to Tokio listener
        let (signal_tx, signal_rx) = async_channel::unbounded::<()>();
        
        let signal_tx_clone = signal_tx.clone();
        
        // Setup synchronous event listener side-by-side with Tokio
        thread::spawn(move || {
            let mut listener = EventListener::new();
            listener.add_active_window_change_handler(move |_| {
                let _ = signal_tx_clone.send_blocking(());
            });
            let _ = listener.start_listener();
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Task 4: IPC Loop logic (Snapshot previous active window before complete state shift)
            let mut last_address: Option<Address> = None;
            
            // Initial fetch
            let _ = refresh_and_send(&sender, &mut last_address).await;
            
            // Loop natively through our bridged signals
            while let Ok(_) = signal_rx.recv().await {
                // If a focus shift occurs, we grab the screencopy for the previously active window immediately
                if let Some(prev) = last_address.clone() {
                    let out_path = format!("{}/{}.png", crate::config::THUMBNAIL_DIR, prev);
                    let _ = crate::backend::screencopy::capture_active_workspace(&out_path).await;
                }
                let _ = refresh_and_send(&sender, &mut last_address).await;
            }
        });
    });
}

async fn refresh_and_send(
    sender: &async_channel::Sender<Vec<WindowData>>, 
    last_address: &mut Option<Address>
) -> Result<(), ()> {
    if let Ok(clients) = Clients::get_async().await {
        let mut windows = Vec::new();
        for client in clients {
            if !client.title.is_empty() && client.mapped {
                windows.push(WindowData {
                    address: client.address.clone(),
                    title: client.title.clone(),
                    class: client.class.clone(),
                });
            }
        }
        let _ = sender.send(windows).await;
    }
    
    // Update baseline
    if let Ok(Some(active)) = Client::get_active_async().await {
        *last_address = Some(active.address);
    }
    Ok(())
}
