mod backend;
mod config;
mod ui;

use gtk4::Application;
use gtk4::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use std::cell::Cell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use tokio::signal::unix::{SignalKind, signal};

fn set_overlay_visibility(window: &gtk4::ApplicationWindow, visible: bool) {
    if visible {
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_visible(true);
        window.present();
        let _ = window.grab_focus();
    } else {
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_visible(false);
    }
}

fn spawn_sigusr1_listener(sender: async_channel::Sender<()>) {
    thread::spawn(move || {
        if let Ok(runtime) = tokio::runtime::Runtime::new() {
            runtime.block_on(async move {
                if let Ok(mut sigusr1) = signal(SignalKind::user_defined1()) {
                    while sigusr1.recv().await.is_some() {
                        let _ = sender.send(()).await;
                    }
                }
            });
        }
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let daemon_mode = args.iter().any(|arg| arg == "--daemon");
    let gtk_args: Vec<&str> = args
        .iter()
        .filter_map(|arg| (arg != "--daemon").then_some(arg.as_str()))
        .collect();
    let (signal_tx, signal_rx) = async_channel::unbounded::<()>();
    spawn_sigusr1_listener(signal_tx);
    let signal_listener_attached = Rc::new(Cell::new(false));

    let app = Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.connect_startup(|_| {
        ui::css::load_css();
        if !Path::new(config::THUMBNAIL_DIR).exists() {
            if let Err(_) = fs::create_dir_all(config::THUMBNAIL_DIR) {
                eprintln!(
                    "Failed to create local tmpfs cache at {}",
                    config::THUMBNAIL_DIR
                );
            }
        }
    });

    let signal_listener_attached_activate = signal_listener_attached.clone();
    app.connect_activate(move |app| {
        if app.active_window().is_none() {
            ui::build_ui(app);
        }

        let Some(window) = app
            .active_window()
            .and_then(|window| window.downcast::<gtk4::ApplicationWindow>().ok())
        else {
            return;
        };

        set_overlay_visibility(&window, !daemon_mode);

        if !signal_listener_attached_activate.get() {
            signal_listener_attached_activate.set(true);
            let signal_window = window.clone();
            let signal_rx = signal_rx.clone();
            gtk4::glib::MainContext::default().spawn_local(async move {
                while signal_rx.recv().await.is_ok() {
                    let visible = signal_window.is_visible();
                    set_overlay_visibility(&signal_window, !visible);
                }
            });
        }
    });

    app.run_with_args(&gtk_args);
}
