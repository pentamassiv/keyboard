use super::submitter::*;
use crate::config::directories;
use crate::config::ui_defaults;
use crate::keyboard;
use crate::keyboard::{EmitUIMsg, KeyEvent, UIMsg};
use gtk::OverlayExt;
use gtk::*;
use relm::Channel;
use std::collections::HashMap;
use std::time::Instant;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

mod dbus;
mod relm_update;
mod relm_widget;
mod ui_manager;
use ui_manager::*;

#[derive(Clone)]
struct Dot {
    x: f64,
    y: f64,
    time: Instant,
}

struct Input {
    input_type: KeyEvent,
    path: Vec<Dot>,
}

pub struct Model {
    keyboard: crate::keyboard::Keyboard,
    input: Input,
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    Press(f64, f64, Instant),
    LongPress(f64, f64, Instant),
    Swipe(f64, f64, Instant),
    Release(f64, f64, Instant),
    Submit(Submission),
    Visible(bool),
    HintPurpose(ContentHint, ContentPurpose),
    ChangeUILayoutView(Option<String>, Option<String>),
    ChangeUIMode(Mode),
    ChangeKBLayoutView(String, String),
    PollEvents,
    UpdateDrawBuffer,
    Quit,
}

pub enum Mode {
    Landscape,
    Portrait,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
struct Gestures {
    _long_press_gesture: GestureLongPress,
    _drag_gesture: GestureDrag,
    _pan_gesture: GesturePan,
}

struct Widgets {
    window: Window,
    draw_handler: relm::DrawHandler<DrawingArea>,
    stack: gtk::Stack,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
pub struct Win {
    pub relm: relm::Relm<Win>,
    model: Model,
    widgets: Widgets,
    _gestures: Gestures,
    ui_manager: UIManager,
    _channel: Channel<Msg>,
}

impl Win {
    fn activate_button(&self, x: f64, y: f64) {
        let (x_rel, y_rel) = self.get_rel_coordinates(x, y);
        let (layout_name, view_name) = &self.model.keyboard.get_view_name();
        if let Some(key_to_activate) =
            self.model
                .keyboard
                .get_closest_key(layout_name, view_name, x_rel, y_rel)
        {
            key_to_activate.activate(self, &self.model.input.input_type);
        }
    }

    fn get_rel_coordinates(&self, x: f64, y: f64) -> (i32, i32) {
        let allocation = self.widgets.stack.get_allocation();
        let (width, height) = (allocation.width, allocation.height);
        let x_rel = (crate::keyboard::RESOLUTIONX as f64 * (x / width as f64)) as i32;
        let y_rel = (crate::keyboard::RESOLUTIONY as f64 * (y / height as f64)) as i32;
        (x_rel, y_rel)
    }

    fn erase_path(&mut self) {
        let context = self.widgets.draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
    }

    fn draw_path(&mut self) {
        self.erase_path();
        if self.model.input.input_type == KeyEvent::Swipe {
            let context = self.widgets.draw_handler.get_context();
            context.set_operator(cairo::Operator::Over);
            context.set_source_rgba(
                ui_defaults::PATHCOLOR.0,
                ui_defaults::PATHCOLOR.1,
                ui_defaults::PATHCOLOR.2,
                ui_defaults::PATHCOLOR.3,
            );
            let max_duration = std::time::Duration::from_millis(ui_defaults::PATHFADINGDURATION);
            for dot in self
                .model
                .input
                .path
                .iter()
                .rev()
                .take(ui_defaults::PATHLENGTH)
            {
                // Only draw the last dots within a certain time period. Works but there would have to be a draw signal in a regular interval to make it look good
                if dot.time.elapsed() < max_duration {
                    context.line_to(dot.x, dot.y);
                } else {
                    break;
                }
            }
            context.set_line_width(ui_defaults::PATHWIDTH);
            context.stroke();
        }
    }
}

// Needed because Rust does not allow implementing a trait for a struct if neighter of them is defined in the scope
// Relm is from the relm crate and EmitUIMsg is from another module
pub struct MessagePipe {
    relm: relm::Relm<crate::user_interface::Win>,
}

impl MessagePipe {
    fn new(relm: relm::Relm<Win>) -> MessagePipe {
        MessagePipe { relm }
    }
}

impl EmitUIMsg for MessagePipe {
    fn emit(&self, message: UIMsg) {
        match message {
            UIMsg::ChangeUILayoutView(layout, view) => {
                self.relm
                    .stream()
                    .emit(Msg::ChangeUILayoutView(layout, view));
            }
            UIMsg::Visible(visable) => {
                self.relm.stream().emit(Msg::Visible(visable));
            }
            UIMsg::HintPurpose(content_hint, content_purpose) => {
                self.relm
                    .stream()
                    .emit(Msg::HintPurpose(content_hint, content_purpose));
            }
        }
    }
}
