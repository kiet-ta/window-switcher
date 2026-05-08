use hyprland::event_listener::EventListener;

fn main() {
    let mut listener = EventListener::new();
    println!("Starting listener...");
    match listener.start_listener() {
        Ok(_) => println!("Listener exited normally"),
        Err(e) => println!("Listener failed: {:?}", e),
    }
}
