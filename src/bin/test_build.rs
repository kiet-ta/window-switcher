use std::collections::HashSet;
use std::process::Command;

#[derive(serde::Deserialize, Debug)]
struct HyprctlWorkspace {
    id: i32,
}

#[derive(serde::Deserialize, Debug)]
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

fn main() {
    let output = Command::new("hyprctl").args(&["clients", "-j"]).output().unwrap();
    let clients: Vec<HyprctlClient> = serde_json::from_slice(&output.stdout).unwrap();
    
    let mut items = Vec::new();
    let active_address = None;

    for client in clients {
        if !client.mapped {
            continue;
        }

        let is_active = active_address == Some(client.address.as_str());
        if client.title.is_empty() && !is_active {
            continue;
        }

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

        items.push((client.address, title));
    }
    
    println!("Built {} items", items.len());
    for item in items {
        println!(" - {:?}", item);
    }
}
