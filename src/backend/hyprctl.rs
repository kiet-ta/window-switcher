use hyprland::data::Clients;
use hyprland::shared::{Address, HyprData};
use std::thread;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WindowData {
    pub address: Address,
    pub title: String,
    pub class: String,
}

pub fn spawn_listener(sender: async_channel::Sender<Vec<WindowData>>) {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
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
        });
    });
}
