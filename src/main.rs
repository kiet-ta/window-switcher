mod config;
mod backend;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;
use std::fs;
use std::path::Path;

fn main() {
    let app = Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.connect_startup(|_| {
        ui::css::load_css();
        if !Path::new(config::THUMBNAIL_DIR).exists() {
            fs::create_dir_all(config::THUMBNAIL_DIR).unwrap_or_else(|_| {
                eprintln!("Failed to create local tmpfs cache at {}", config::THUMBNAIL_DIR);
            });
        }
    });

    app.connect_activate(ui::build_ui);
    app.run();
}
