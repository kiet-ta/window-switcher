use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, EventControllerKey, FlowBox};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::hyprctl::FocusController;
use crate::config::{
    DEFAULT_COLUMNS, MAX_COLUMNS, UI_CARD_WIDTH, UI_GRID_SPACING, UI_OUTER_MARGIN,
};

fn hide_overlay(window: &ApplicationWindow) {
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_visible(false);
}

fn focus_selected(
    focus_controller: &FocusController,
    selected_address: &Rc<RefCell<Option<String>>>,
) {
    if let Some(address) = selected_address.borrow().as_deref() {
        focus_controller.enqueue(address);
    }
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

fn apply_selection(
    flow_box: &FlowBox,
    rendered_order: &[String],
    selected_address: &Rc<RefCell<Option<String>>>,
    index: usize,
) {
    if let Some(address) = rendered_order.get(index) {
        *selected_address.borrow_mut() = Some(address.clone());
        if let Some(child) = flow_box.child_at_index(index as i32) {
            child.grab_focus();
            flow_box.select_child(&child);
        }
    }
}

pub fn bind_keys(
    window: &ApplicationWindow,
    flow_box: &FlowBox,
    rendered_order: Rc<RefCell<Vec<String>>>,
    selected_address: Rc<RefCell<Option<String>>>,
    focus_controller: FocusController,
) {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let window_pressed = window.clone();
    let flow_box_pressed = flow_box.clone();
    let rendered_order_pressed = rendered_order.clone();
    let selected_address_pressed = selected_address.clone();
    let focus_controller_pressed = focus_controller.clone();

    controller.connect_key_pressed(move |_, key, _, _| {
        let rendered_order = rendered_order_pressed.borrow();
        let item_count = rendered_order.len();
        if item_count == 0 {
            return match key {
                Key::Escape | Key::Return => {
                    hide_overlay(&window_pressed);
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            };
        }

        let columns = estimated_columns(window_pressed.width(), item_count);
        let current_selection = selected_address_pressed.borrow().clone();
        let mut index = current_selection
            .as_ref()
            .and_then(|address| rendered_order.iter().position(|candidate| candidate == address))
            .unwrap_or(0);
        let mut handled = true;

        match key {
            Key::Right | Key::Tab => {
                index = (index + 1) % item_count;
            }
            Key::Left => {
                index = index.saturating_sub(1);
            }
            Key::Down => {
                index = (index + columns).min(item_count - 1);
            }
            Key::Up => {
                index = index.saturating_sub(columns);
            }
            Key::Return => {
                focus_selected(&focus_controller_pressed, &selected_address_pressed);
                hide_overlay(&window_pressed);
            }
            Key::Escape => {
                hide_overlay(&window_pressed);
            }
            _ => {
                handled = false;
            }
        }

        if handled {
            apply_selection(
                &flow_box_pressed,
                rendered_order.as_slice(),
                &selected_address_pressed,
                index,
            );
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    let window_released = window.clone();
    let selected_address_released = selected_address.clone();
    controller.connect_key_released(move |_, key, _, _| {
        if matches!(key, Key::Alt_L | Key::Alt_R) {
            focus_selected(&focus_controller, &selected_address_released);
            hide_overlay(&window_released);
        }
    });

    window.add_controller(controller);
}
