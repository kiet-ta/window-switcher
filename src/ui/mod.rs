pub mod css;
pub mod input;

use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, FlowBox, FlowBoxChild, Frame, Image,
    Label, Orientation, Picture, PolicyType, ScrolledWindow, Stack,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::backend::hyprctl::{
    thumbnail_path, spawn_backend, ThumbnailState, UiSnapshot, UiStatus, WindowData,
};
use crate::config::{
    DEFAULT_COLUMNS, MAX_COLUMNS, UI_CARD_HEIGHT, UI_CARD_WIDTH, UI_GRID_SPACING,
    UI_OUTER_MARGIN, UI_THUMBNAIL_HEIGHT, UI_THUMBNAIL_WIDTH,
};

struct WindowCard {
    child: FlowBoxChild,
    frame: Frame,
    title: Label,
    meta: Label,
    state: Label,
    picture: Picture,
    icon: Image,
    preview_stack: Stack,
}

impl WindowCard {
    fn new() -> Self {
        let frame = Frame::builder()
            .css_classes(vec!["window-frame".to_string()])
            .focusable(true)
            .build();
        frame.set_size_request(UI_CARD_WIDTH, UI_CARD_HEIGHT);

        let item_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .halign(Align::Fill)
            .valign(Align::Fill)
            .build();

        let preview_stack = Stack::builder()
            .css_classes(vec!["thumbnail-stack".to_string()])
            .halign(Align::Fill)
            .build();
        preview_stack.set_size_request(UI_THUMBNAIL_WIDTH, UI_THUMBNAIL_HEIGHT);

        let picture = Picture::builder()
            .can_shrink(true)
            .keep_aspect_ratio(true)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();
        picture.set_size_request(UI_THUMBNAIL_WIDTH, UI_THUMBNAIL_HEIGHT);

        let icon = Image::from_icon_name("application-x-executable");
        icon.set_pixel_size(72);
        icon.set_halign(Align::Center);
        icon.set_valign(Align::Center);

        let fallback_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .css_classes(vec!["thumbnail-fallback".to_string()])
            .halign(Align::Fill)
            .valign(Align::Fill)
            .build();
        fallback_box.set_size_request(UI_THUMBNAIL_WIDTH, UI_THUMBNAIL_HEIGHT);
        fallback_box.append(&icon);

        preview_stack.add_named(&picture, Some("preview"));
        preview_stack.add_named(&fallback_box, Some("fallback"));
        preview_stack.set_visible_child_name("fallback");
        item_box.append(&preview_stack);

        let title = Label::builder()
            .css_classes(vec!["window-title".to_string()])
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title.set_wrap(false);
        title.set_max_width_chars(28);

        let meta = Label::builder()
            .css_classes(vec!["window-meta".to_string()])
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        meta.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        meta.set_wrap(false);
        meta.set_max_width_chars(28);

        let state = Label::builder()
            .css_classes(vec!["window-state".to_string()])
            .halign(Align::Start)
            .xalign(0.0)
            .build();
        state.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        state.set_wrap(false);
        state.set_max_width_chars(28);

        item_box.append(&title);
        item_box.append(&meta);
        item_box.append(&state);
        frame.set_child(Some(&item_box));

        let child = FlowBoxChild::new();
        child.set_focusable(true);
        child.set_child(Some(&frame));

        Self {
            child,
            frame,
            title,
            meta,
            state,
            picture,
            icon,
            preview_stack,
        }
    }

    fn update(&self, item: &WindowData, selected: bool) {
        self.title.set_text(&item.title);
        self.title.set_tooltip_text(Some(&item.title));
        self.frame
            .set_tooltip_text(Some(&format!("{} ({})", item.title, item.address)));

        let meta = describe_window(item);
        self.meta.set_text(&meta);
        self.meta.set_tooltip_text(Some(&meta));

        let icon_name = if item.class.is_empty() {
            "application-x-executable"
        } else {
            item.class.as_str()
        };
        self.icon.set_icon_name(Some(icon_name));

        match item.thumbnail_state {
            ThumbnailState::Ready => {
                let file = gtk4::gio::File::for_path(thumbnail_path(&item.address));
                self.picture.set_file(Some(&file));
                self.preview_stack.set_visible_child_name("preview");
                self.state
                    .set_text(if item.is_active { "Active window" } else { "Preview ready" });
                self.frame.remove_css_class("thumbnail-pending");
                self.frame.remove_css_class("thumbnail-missing");
                self.frame.remove_css_class("thumbnail-failed");
            }
            ThumbnailState::Pending => {
                self.preview_stack.set_visible_child_name("fallback");
                self.state.set_text("Preview loading");
                self.frame.add_css_class("thumbnail-pending");
                self.frame.remove_css_class("thumbnail-missing");
                self.frame.remove_css_class("thumbnail-failed");
            }
            ThumbnailState::Missing => {
                self.preview_stack.set_visible_child_name("fallback");
                self.state.set_text("Preview unavailable");
                self.frame.remove_css_class("thumbnail-pending");
                self.frame.add_css_class("thumbnail-missing");
                self.frame.remove_css_class("thumbnail-failed");
            }
            ThumbnailState::Failed => {
                self.preview_stack.set_visible_child_name("fallback");
                self.state.set_text("Preview capture failed");
                self.frame.remove_css_class("thumbnail-pending");
                self.frame.remove_css_class("thumbnail-missing");
                self.frame.add_css_class("thumbnail-failed");
            }
        }

        if selected {
            self.frame.add_css_class("selected-window");
        } else {
            self.frame.remove_css_class("selected-window");
        }

        if item.is_active {
            self.frame.add_css_class("active-window");
        } else {
            self.frame.remove_css_class("active-window");
        }
    }
}

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
    window.set_focusable(true);

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(UI_OUTER_MARGIN)
        .margin_bottom(UI_OUTER_MARGIN)
        .margin_start(UI_OUTER_MARGIN)
        .margin_end(UI_OUTER_MARGIN)
        .css_classes(vec!["overlay-root".to_string()])
        .build();

    let header = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(vec!["main-container".to_string()])
        .build();

    let title = Label::builder()
        .label("Window Switcher")
        .css_classes(vec!["title-label".to_string()])
        .halign(Align::Start)
        .xalign(0.0)
        .build();
    header.append(&title);

    let status_label = Label::builder()
        .css_classes(vec!["status-label".to_string()])
        .halign(Align::Start)
        .xalign(0.0)
        .build();
    header.append(&status_label);
    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .css_classes(vec!["grid-scroll".to_string()])
        .build();

    let flow_box = FlowBox::builder()
        .valign(Align::Start)
        .halign(Align::Center)
        .selection_mode(gtk4::SelectionMode::Browse)
        .max_children_per_line(DEFAULT_COLUMNS as u32)
        .min_children_per_line(1)
        .column_spacing(UI_GRID_SPACING as u32)
        .row_spacing(UI_GRID_SPACING as u32)
        .activate_on_single_click(false)
        .css_classes(vec!["window-grid".to_string()])
        .build();

    scroller.set_child(Some(&flow_box));
    header.append(&scroller);
    root.append(&header);
    window.set_child(Some(&root));

    let rendered_order = Rc::new(RefCell::new(Vec::<String>::new()));
    let selected_address = Rc::new(RefCell::new(None::<String>));
    let card_cache = Rc::new(RefCell::new(HashMap::<String, WindowCard>::new()));

    let (snapshot_tx, snapshot_rx) = async_channel::unbounded::<UiSnapshot>();
    let focus_controller = spawn_backend(snapshot_tx);

    let flow_box_for_updates = flow_box.clone();
    let window_for_updates = window.clone();
    let rendered_order_for_updates = rendered_order.clone();
    let selected_address_for_updates = selected_address.clone();
    let card_cache_for_updates = card_cache.clone();
    let status_label_for_updates = status_label.clone();

    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Ok(snapshot) = snapshot_rx.recv().await {
            apply_snapshot(
                &window_for_updates,
                &flow_box_for_updates,
                &status_label_for_updates,
                &rendered_order_for_updates,
                &selected_address_for_updates,
                &card_cache_for_updates,
                snapshot,
            );
        }
    });

    input::bind_keys(
        &window,
        &flow_box,
        rendered_order,
        selected_address,
        focus_controller,
    );
}

fn apply_snapshot(
    window: &ApplicationWindow,
    flow_box: &FlowBox,
    status_label: &Label,
    rendered_order: &Rc<RefCell<Vec<String>>>,
    selected_address: &Rc<RefCell<Option<String>>>,
    card_cache: &Rc<RefCell<HashMap<String, WindowCard>>>,
    snapshot: UiSnapshot,
) {
    let new_order: Vec<String> = snapshot
        .items
        .iter()
        .map(|item| item.address.clone())
        .collect();
    let previous_order = rendered_order.borrow().clone();
    let order_changed = previous_order != new_order;
    let present_addresses: HashSet<String> = new_order.iter().cloned().collect();

    let stale_addresses: Vec<String> = card_cache
        .borrow()
        .keys()
        .filter(|address| !present_addresses.contains(*address))
        .cloned()
        .collect();
    for address in stale_addresses {
        let removed = { card_cache.borrow_mut().remove(&address) };
        if let Some(card) = removed {
            flow_box.remove(&card.child);
        }
    }

    {
        let mut cards = card_cache.borrow_mut();
        for item in &snapshot.items {
            cards
                .entry(item.address.clone())
                .or_insert_with(WindowCard::new);
        }
    }

    if order_changed {
        let cards = card_cache.borrow();
        for address in &previous_order {
            if let Some(card) = cards.get(address) {
                flow_box.remove(&card.child);
            }
        }
        for address in &new_order {
            if let Some(card) = cards.get(address) {
                flow_box.insert(&card.child, -1);
            }
        }
    } else if previous_order.is_empty() {
        let cards = card_cache.borrow();
        for address in &new_order {
            if let Some(card) = cards.get(address) {
                flow_box.insert(&card.child, -1);
            }
        }
    }

    *rendered_order.borrow_mut() = new_order.clone();
    apply_layout(flow_box, window, new_order.len());

    let current_selection = selected_address.borrow().clone();
    let selection_to_apply = if !order_changed {
        current_selection
            .filter(|address| present_addresses.contains(address))
            .or(snapshot.selected_address.clone())
    } else {
        snapshot.selected_address.clone()
    }
    .or_else(|| new_order.first().cloned());

    *selected_address.borrow_mut() = selection_to_apply.clone();

    {
        let cards = card_cache.borrow();
        for item in &snapshot.items {
            if let Some(card) = cards.get(&item.address) {
                card.update(
                    item,
                    selection_to_apply.as_deref() == Some(item.address.as_str()),
                );
            }
        }
    }

    update_status_label(status_label, &snapshot.status, snapshot.items.is_empty());
    flow_box.set_visible(!snapshot.items.is_empty());

    if let Some(selected_address) = selection_to_apply {
        if let Some(index) = new_order.iter().position(|address| address == &selected_address) {
            if let Some(child) = flow_box.child_at_index(index as i32) {
                flow_box.select_child(&child);
                child.grab_focus();
            }
        }
    }
}

fn apply_layout(flow_box: &FlowBox, window: &ApplicationWindow, item_count: usize) {
    let columns = estimated_columns(window.width(), item_count);
    flow_box.set_max_children_per_line(columns as u32);
}

fn estimated_columns(window_width: i32, item_count: usize) -> usize {
    if item_count == 0 {
        return 1;
    }

    if window_width <= 0 {
        return DEFAULT_COLUMNS.min(item_count).max(1);
    }

    let usable_width = (window_width - (UI_OUTER_MARGIN * 2)).max(UI_CARD_WIDTH);
    let card_span = (UI_CARD_WIDTH + UI_GRID_SPACING).max(1);
    let estimated = (usable_width / card_span).max(1) as usize;
    estimated.clamp(1, MAX_COLUMNS.min(item_count))
}

fn update_status_label(label: &Label, status: &UiStatus, is_empty: bool) {
    let text = match status {
        UiStatus::Loading => "Loading windows and previews...",
        UiStatus::Ready if is_empty => "No windows available.",
        UiStatus::Ready => "Use arrows or Tab to move, Enter to switch.",
        UiStatus::Empty => "No windows available.",
        UiStatus::BackendDegraded => "Limited backend data. Fallback metadata is still usable.",
    };

    label.set_text(text);
}

fn describe_window(item: &WindowData) -> String {
    let mut segments = Vec::new();

    if item.is_active {
        segments.push("Active".to_string());
    }

    if let Some(monitor_name) = item.monitor_name.as_ref() {
        segments.push(format!("Monitor {monitor_name}"));
    }

    if let Some(workspace_id) = item.workspace_id {
        segments.push(format!("Workspace {workspace_id}"));
    }

    if segments.is_empty() {
        segments.push(item.class.clone());
    }

    segments.join(" / ")
}
