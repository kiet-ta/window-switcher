use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, FlowBox, Frame, Image, Label, Orientation, Box as GtkBox, EventControllerKey};
use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

const APP_ID: &str = "com.antigravity.window_switcher";
const THUMBNAIL_DIR: &str = "/tmp/switcher-thumbnails";
const COLUMNS: usize = 4;
const MOCK_ITEMS: usize = 5;

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        load_css();
        if !Path::new(THUMBNAIL_DIR).exists() {
            fs::create_dir_all(THUMBNAIL_DIR).unwrap_or_else(|_| {
                eprintln!("Failed to create local tmpfs cache at {}", THUMBNAIL_DIR);
            });
        }
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Switcher")
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 100);
    window.set_margin(Edge::Bottom, 100);
    window.set_margin(Edge::Left, 200);
    window.set_margin(Edge::Right, 200);

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .css_classes(vec!["main-container".to_string()])
        .build();

    let title = Label::builder()
        .label("Window Switcher")
        .css_classes(vec!["title-label".to_string()])
        .build();
    
    container.append(&title);

    let flow_box = FlowBox::builder()
        .valign(gtk4::Align::Start)
        .halign(gtk4::Align::Center)
        .selection_mode(gtk4::SelectionMode::Browse)
        .max_children_per_line(COLUMNS as u32)
        .min_children_per_line(1)
        .column_spacing(20)
        .row_spacing(20)
        .build();

    for i in 0..MOCK_ITEMS {
        let frame = Frame::builder()
            .css_classes(vec!["window-frame".to_string()])
            .focusable(true) // Ensure frame can grab physical UI focus
            .build();
            
        let item_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();
        
        let icon = Image::builder()
            .icon_name("image-missing")
            .pixel_size(128)
            .build();
            
        let label = Label::builder()
            .label(format!("Mock Window {}", i + 1))
            .css_classes(vec!["window-title".to_string()])
            .build();
            
        item_box.append(&icon);
        item_box.append(&label);
        frame.set_child(Some(&item_box));
        flow_box.insert(&frame, -1);
    }

    container.append(&flow_box);
    window.set_child(Some(&container));
    
    // Phase 3: Spatial Navigation (Grid Math Logic)
    let active_index = Rc::new(Cell::new(0));
    
    // Select the first item initially
    if let Some(first_child) = flow_box.child_at_index(0) {
        flow_box.select_child(&first_child);
    }
    
    let controller = EventControllerKey::new();
    let flow_box_clone = flow_box.clone();
    let window_clone = window.clone();
    
    controller.connect_key_pressed(move |_, key, _, _| {
        let mut idx = active_index.get();
        let total = MOCK_ITEMS;
        let mut handled = true;
        
        match key {
            Key::Right => {
                if idx + 1 < total { idx += 1; }
            }
            Key::Left => {
                if idx > 0 { idx -= 1; }
            }
            Key::Down => {
                if idx + COLUMNS < total { 
                    idx += COLUMNS; 
                } else { 
                    idx = total - 1; // Snap to the last available item 
                }
            }
            Key::Up => {
                if idx >= COLUMNS { 
                    idx -= COLUMNS; 
                } else { 
                    idx = 0; // Snap to the first item
                }
            }
            Key::Return => {
                println!("Window selected at index: {}", idx);
                // Phase 4 will implement actual dispatch to Hyprland here
                window_clone.close();
            }
            Key::Escape => {
                window_clone.close();
            }
            _ => { handled = false; }
        }
        
        if handled {
            active_index.set(idx);
            if let Some(child) = flow_box_clone.child_at_index(idx as i32) {
                child.grab_focus();
                flow_box_clone.select_child(&child);
            }
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });
    
    window.add_controller(controller);
    window.present();
}

fn load_css() {
    let provider = CssProvider::new();
    let css_data = r#"
        window {
            background-color: rgba(15, 15, 20, 0.85); 
            border-radius: 15px;
        }
        
        .main-container {
            padding: 30px;
        }

        .title-label {
            font-size: 24pt;
            font-weight: bold;
            color: white;
        }

        .window-frame {
            background-color: rgba(40, 40, 50, 0.9);
            border-radius: 10px;
            padding: 15px;
            border: 2px solid transparent;
            transition: all 0.2s ease-in-out;
        }

        /* Use both hover and GTK actual focus state for visual feedback */
        .window-frame:hover, .window-frame:focus-within {
            border: 2px solid #00ffcc;
            background-color: rgba(60, 60, 80, 0.9);
            box-shadow: 0 0 10px rgba(0, 255, 204, 0.5);
        }

        .window-title {
            color: white;
            font-size: 14pt;
            font-weight: 500;
        }
    "#;
    
    provider.load_from_data(css_data);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
