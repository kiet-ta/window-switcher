pub mod css;
pub mod input;

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, FlowBox, Frame, Image, Label, Orientation,
    Picture,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::backend::hyprctl::WindowData;
use crate::config::{COLUMNS, THUMBNAIL_DIR};

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprland Switcher")
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::None);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 100);
    window.set_margin(Edge::Bottom, 100);
    window.set_margin(Edge::Left, 200);
    window.set_margin(Edge::Right, 200);
    window.set_focusable(true);

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

    container.append(&flow_box);
    window.set_child(Some(&container));

    let active_index = Rc::new(RefCell::new(0));
    let window_list: Rc<RefCell<Vec<WindowData>>> = Rc::new(RefCell::new(Vec::new()));

    let (sender, receiver) = async_channel::unbounded();
    crate::backend::hyprctl::spawn_listener(sender);

    let flow_box_clone = flow_box.clone();
    let window_list_rx = window_list.clone();

    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Ok(windows) = receiver.recv().await {
            *window_list_rx.borrow_mut() = windows.clone();

            while let Some(child) = flow_box_clone.child_at_index(0) {
                flow_box_clone.remove(&child);
            }

            for window_data in windows.iter() {
                let frame = Frame::builder()
                    .css_classes(vec!["window-frame".to_string()])
                    .focusable(true)
                    .build();

                let item_box = GtkBox::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(10)
                    .halign(gtk4::Align::Center)
                    .build();

                let image_path = format!("{}/{}.png", THUMBNAIL_DIR, window_data.address);
                if Path::new(&image_path).exists() {
                    let file = gtk4::gio::File::for_path(image_path);
                    let pic = Picture::builder()
                        .file(&file)
                        .can_shrink(true)
                        .keep_aspect_ratio(true)
                        .height_request(128)
                        .width_request(128)
                        .halign(gtk4::Align::Center)
                        .build();
                    item_box.append(&pic);
                } else {
                    let icon_name = if window_data.class.is_empty() {
                        "application-x-executable"
                    } else {
                        &window_data.class
                    };
                    let icon = Image::from_icon_name(icon_name);
                    icon.set_icon_size(gtk4::IconSize::Large);
                    icon.set_pixel_size(128);
                    icon.set_halign(gtk4::Align::Center);
                    item_box.append(&icon);
                }

                let label = Label::builder()
                    .label(window_data.title.chars().take(20).collect::<String>())
                    .css_classes(vec!["window-title".to_string()])
                    .halign(gtk4::Align::Center)
                    .build();

                item_box.append(&label);
                frame.set_child(Some(&item_box));
                flow_box_clone.insert(&frame, -1);
            }

            if let Some(first_child) = flow_box_clone.child_at_index(0) {
                first_child.grab_focus();
                flow_box_clone.select_child(&first_child);
            }
        }
    });

    input::bind_keys(&window, &flow_box, window_list, active_index);
}
