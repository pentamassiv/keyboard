#[cfg(feature = "gesture")]
use crate::config::path_defaults;
use crate::keyboard::{Interaction, TapMotion};
use gtk::WidgetExt;
use relm::Channel;
use std::collections::{HashMap, HashSet};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

mod gesture_handler;
mod relm_update;
mod relm_widget;
mod ui_manager;
use gesture_handler::{GestureModel, GestureSignal};
use ui_manager::UIManager;

pub struct Model {
    gesture: GestureModel,
    latched_keys: HashSet<String>,
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    GestureSignal(f64, f64, GestureSignal),
    Interaction((f64, f64), Interaction),
    ButtonInteraction(String, TapMotion),
    LatchingButtonInteraction(String),
    ReleaseAllButtions,
    OpenPopup(String),
    SubmitText(String),
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

#[derive(Copy, Debug, Clone)]
pub enum Orientation {
    Landscape,
    Portrait,
}

#[derive(Debug, Clone)]
struct Gestures {
    long_press_gesture: gtk::GestureLongPress,
    drag_gesture: gtk::GestureDrag,
}

struct Widgets {
    window: gtk::Window,
    _overlay: gtk::Overlay,
    _draw_handler: relm::DrawHandler<gtk::DrawingArea>,
    stack: gtk::Stack,
}

pub struct Win {
    pub relm: relm::Relm<Win>,
    model: Model,
    keyboard: crate::keyboard::Keyboard,
    key_refs: HashMap<(String, String, String), (gtk::ToggleButton, Option<gtk::Popover>)>,
    widgets: Widgets,
    gestures: Gestures,
    ui_manager: UIManager,
    _channel: Channel<Msg>, // The channel needs to be saved to prevent dropping it and thus closing the channel
}

impl Win {
    fn get_rel_coordinates(&self, x: f64, y: f64) -> (i32, i32) {
        let allocation = self.widgets.stack.get_allocation();
        let (width, height) = (allocation.width, allocation.height);
        let x_rel = (crate::keyboard::RESOLUTIONX as f64 * (x / width as f64)) as i32;
        let y_rel = (crate::keyboard::RESOLUTIONY as f64 * (y / height as f64)) as i32;
        info!("The relative coordinate is x: {}, y: {}", x_rel, y_rel);
        (x_rel, y_rel)
    }

    #[cfg(feature = "gesture")]
    fn erase_path(&mut self) {
        let context = self.widgets._draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
        info!("Path of gesture was erased");
    }

    #[cfg(feature = "gesture")]
    fn draw_path(&mut self) {
        self.erase_path();
        let context = self.widgets._draw_handler.get_context();
        context.set_operator(cairo::Operator::Over);
        context.set_source_rgba(
            path_defaults::PATHCOLOR.0,
            path_defaults::PATHCOLOR.1,
            path_defaults::PATHCOLOR.2,
            path_defaults::PATHCOLOR.3,
        );
        let max_duration = std::time::Duration::from_millis(path_defaults::PATHFADINGDURATION);
        for dot in self
            .model
            .gesture
            .get_swipe_path()
            .iter()
            .rev()
            .take(path_defaults::PATHLENGTH)
        {
            // Only draw the last dots within a certain time period. Works but there would have to be a draw signal in a regular interval to make it look good
            if dot.time.elapsed() < max_duration {
                context.line_to(dot.x, dot.y);
            } else {
                break;
            }
        }
        context.set_line_width(path_defaults::PATHWIDTH);
        context.stroke();
        info!("Path of gesture was drawn");
    }
}
