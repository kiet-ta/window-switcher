use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, EventControllerKey, FlowBox};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::hyprctl::WindowData;
use crate::config::COLUMNS;

fn focus_window_by_index(window_list: &Rc<RefCell<Vec<WindowData>>>, idx: usize) {
    if let Some(win) = window_list.borrow().get(idx) {
        let address_str = format!("address:{}", win.address);
        let _ = std::process::Command::new("hyprctl")
            .args(&["dispatch", "focuswindow", &address_str])
            .spawn();
    }
}

fn hide_overlay(window: &ApplicationWindow) {
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_visible(false);
}

pub fn bind_keys(
    window: &ApplicationWindow,
    flow_box: &FlowBox,
    window_list: Rc<RefCell<Vec<WindowData>>>,
    active_index: Rc<RefCell<usize>>,
) {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let window_pressed = window.clone();
    let flow_box_pressed = flow_box.clone();
    let window_list_pressed = window_list.clone();
    let active_index_pressed = active_index.clone();

    controller.connect_key_pressed(move |_, key, _, _| {
        let mut idx = *active_index_pressed.borrow();
        let list_len = window_list_pressed.borrow().len();
        if list_len == 0 {
            return match key {
                Key::Escape | Key::Return => {
                    hide_overlay(&window_pressed);
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            };
        }

        let mut handled = true;

        match key {
            Key::Right | Key::Tab => {
                idx = (idx + 1) % list_len;
            }
            Key::Left => {
                if idx > 0 {
                    idx -= 1;
                }
            }
            Key::Down => {
                // Task 2: Fix 2D Spatial bounds check.
                // Only move down if there is actually a window directly below.
                if idx + COLUMNS < list_len {
                    idx += COLUMNS;
                }
            }
            Key::Up => {
                if idx >= COLUMNS {
                    idx -= COLUMNS;
                }
            }
            Key::Return => {
                focus_window_by_index(&window_list_pressed, idx);
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
            *active_index_pressed.borrow_mut() = idx;
            if let Some(child) = flow_box_pressed.child_at_index(idx as i32) {
                child.grab_focus();
                flow_box_pressed.select_child(&child);
            }
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    let window_released = window.clone();
    let window_list_released = window_list.clone();
    let active_index_released = active_index.clone();
    controller.connect_key_released(move |_, key, _, _| {
        if matches!(key, Key::Alt_L | Key::Alt_R) {
            let idx = *active_index_released.borrow();
            focus_window_by_index(&window_list_released, idx);
            hide_overlay(&window_released);
        }
    });

    window.add_controller(controller);
}
