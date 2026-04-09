use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, EventControllerKey, FlowBox};
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::hyprctl::{self, WindowData};
use crate::config::COLUMNS;

pub fn bind_keys(
    window: &ApplicationWindow,
    flow_box: &FlowBox,
    window_list: Rc<RefCell<Vec<WindowData>>>,
    active_index: Rc<RefCell<usize>>,
) {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let window_clone = window.clone();
    let flow_box_controller_ref = flow_box.clone();

    controller.connect_key_pressed(move |_, key, _, _| {
        let mut idx = *active_index.borrow();
        let list_len = window_list.borrow().len();
        if list_len == 0 {
            return match key {
                Key::Escape | Key::Return => {
                    window_clone.close();
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            };
        }

        let mut handled = true;

        match key {
            Key::Right => {
                if idx + 1 < list_len {
                    idx += 1;
                }
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
                if let Some(win) = window_list.borrow().get(idx) {
                    hyprctl::queue_focus_window(win.address.to_string());
                }
                window_clone.close();
            }
            Key::Escape => {
                window_clone.close();
            }
            _ => {
                handled = false;
            }
        }

        if handled {
            *active_index.borrow_mut() = idx;
            if let Some(child) = flow_box_controller_ref.child_at_index(idx as i32) {
                child.grab_focus();
                flow_box_controller_ref.select_child(&child);
            }
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(controller);
}
