use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    let xdg = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let path = format!("{}/hypr/{}/.socket2.sock", xdg, sig);
    
    if let Ok(stream) = UnixStream::connect(&path) {
        println!("Connected to {}", path);
    } else {
        println!("Failed to connect");
    }
}
