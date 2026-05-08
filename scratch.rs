use hyprland::event_listener::EventListener;

fn main() {
    let mut listener = EventListener::new();
    listener.add_window_open_handler(|_| {});
    listener.add_window_close_handler(|_| {});
    listener.add_window_title_change_handler(|_| {});
}
