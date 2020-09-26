use super::submitter::*;
#[cfg(feature = "suggestions")]
use crate::config::ui_defaults;
use crate::keyboard::input_handler::InputType;
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

// Is only read when gestures feature is turned on
struct Point {
    x: f64,
    y: f64,
    time: Instant,
}

pub struct Model {
    path: Vec<Point>,
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    Input((f64, f64), InputType),
    ButtonInteraction(String, KeyMotion),

    #[cfg(feature = "suggestions")]
    Submit(Submission),
    Visible(bool),
    HintPurpose(ContentHint, ContentPurpose),
    ChangeUILayoutView(Option<String>, Option<String>),
    ChangeUIOrientation(Orientation),
    ChangeKBLayoutView(String, String),
    PollEvents,
    #[cfg(feature = "gesture")]
    UpdateDrawBuffer,
    Quit,
}

pub enum Orientation {
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
    keyboard: crate::keyboard::Keyboard,
    key_refs: HashMap<(String, String, String), ToggleButton>,
    widgets: Widgets,
    _gestures: Gestures,
    ui_manager: UIManager,
    _channel: Channel<Msg>,
}

impl Win {
    fn get_rel_coordinates(&self, x: f64, y: f64) -> (i32, i32) {
        let allocation = self.widgets.stack.get_allocation();
        let (width, height) = (allocation.width, allocation.height);
        let x_rel = (crate::keyboard::RESOLUTIONX as f64 * (x / width as f64)) as i32;
        let y_rel = (crate::keyboard::RESOLUTIONY as f64 * (y / height as f64)) as i32;
        (x_rel, y_rel)
    }

    #[cfg(feature = "gesture")]
    fn erase_path(&mut self) {
        let context = self.widgets.draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
    }

    #[cfg(feature = "gesture")]
    fn draw_path(&mut self) {
        self.erase_path();
        let context = self.widgets.draw_handler.get_context();
        context.set_operator(cairo::Operator::Over);
        context.set_source_rgba(
            ui_defaults::PATHCOLOR.0,
            ui_defaults::PATHCOLOR.1,
            ui_defaults::PATHCOLOR.2,
            ui_defaults::PATHCOLOR.3,
        );
        let max_duration = std::time::Duration::from_millis(ui_defaults::PATHFADINGDURATION);
        for dot in self.model.path.iter().rev().take(ui_defaults::PATHLENGTH) {
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
