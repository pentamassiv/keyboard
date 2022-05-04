// Imports from other crates
#[cfg(feature = "suggestions")]
use gtk::prelude::{ButtonExt, WidgetExt};
use relm::Channel;
use std::collections::HashMap;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

// Imports from other modules
#[cfg(feature = "gesture")]
use crate::config::path_defaults;
use crate::keyboard::TapMotion;

// Modules
mod gesture_handler;
mod relm_update;
mod relm_widget;
mod ui_manager;
use gesture_handler::{GestureModel, GestureSignal};
use ui_manager::UIManager;

/// Saves all relevant information needed to display the user interface
pub struct Model {
    gesture: GestureModel,
}

/// Messages that can be sent to initiate an update of the user interface or react to the users actions
#[derive(relm_derive::Msg)]
pub enum Msg {
    // Contains the coordinates and the type of gesture signal. This message is sent when the user taps or swipes on the keyboard.
    // The raw signals need to be converted to an 'Interaction' before they can get sent to the keyboard module
    GestureSignal(f64, f64, GestureSignal),
    // Contains the layout, view and button_id to identify the button to eighter release or press. This is for the visual feedback only. The buttons do NOT do anything.
    ButtonInteraction(String, String, String, TapMotion),
    // Contains the id of the button which will open its popover
    OpenPopup(String),
    // Contains a string that will be submitted by the keyboard
    SubmitText(String, bool),
    // Updates the suggestions
    #[cfg(feature = "suggestions")]
    Suggestions(Vec<String>),
    // Contains the value the visibility of the user interface is supposed to be set to
    SetVisibility(bool),
    // Contains the ContentHint and ContentPurpose the user_interface is supposed to be set to. This is not implemented yet but in the future, it could change the layout
    HintPurpose(ContentHint, ContentPurpose),
    // Contains the name of the layout and/or view the user interface should change to
    ChangeUILayoutView(Option<String>, Option<String>),
    // Contains the orientation the user interface should change to
    ChangeUIOrientation(Orientation),
    // Contains the name of the layout and view the keyboard struct should change to
    ChangeKBLayoutView(String, String),
    // Poll events from the submitter (needed to get wayland events)
    PollEvents,
    #[cfg(feature = "gesture")]
    // Update the drawn path of a gesture
    UpdateDrawBuffer,
    // End the application
    Quit,
}

#[derive(Copy, Debug, Clone)]
/// Orientation of the user interface
pub enum Orientation {
    // Device is held horizontally (like you would hold it to take a picture of a landscape)
    Landscape,
    // Device is held vertically (like you would hold it to take a selfie)
    Portrait,
}

#[derive(Debug, Clone)]
/// Contains the gesture handler to recognize a long press and a drag
struct Gestures {
    long_press_gesture: gtk::GestureLongPress,
    drag_gesture: gtk::GestureDrag,
}

#[cfg(feature = "suggestions")]
#[derive(Debug, Clone)]
/// Contains the buttons that display the suggestions
struct Suggestions {
    left: gtk::Button,
    center: gtk::Button,
    right: gtk::Button,
}

/// Contains all widgets that need to get accessed
struct Widgets {
    window: gtk::Window,
    _overlay: gtk::Overlay,
    _draw_handler: relm::DrawHandler<gtk::DrawingArea>,
    #[cfg(feature = "suggestions")]
    suggestions: Suggestions,
    stack: gtk::Stack,
    buttons: HashMap<(String, String, String), (gtk::ToggleButton, Option<gtk::Popover>)>,
}

/// Contains all structs needed for the user interface
pub struct Win {
    pub relm: relm::Relm<Win>,
    model: Model,
    keyboard: crate::keyboard::Keyboard,
    widgets: Widgets,
    gestures: Gestures,
    ui_manager: UIManager,
    _channel: Channel<Msg>, // The channel is used to receive messages from other threads like the one the dbus_server is running in.
                            // It needs to be saved to prevent dropping it and thus closing the channel.
}

impl Win {
    /// Converts the given absolute coordinates to coordinates relative to the gtk::Stack's width and height.
    /// This is done to abstract the actual dimensions of the user interface and don't have to recalculate the keys locations each time the size of the user interface changes
    fn get_rel_coordinates(&self, x: f64, y: f64) -> (f64, f64) {
        // Get width and height of the gtk::Stack that is used to display the button rows
        let allocation = self.widgets.stack.allocation();
        let (width, height) = (allocation.width(), allocation.height());
        // Calculate the relative coordinates
        let x_rel = x / width as f64;
        let y_rel = y / height as f64;
        info!("The relative coordinate is x: {}, y: {}", x_rel, y_rel);
        (x_rel, y_rel)
    }

    #[cfg(feature = "gesture")]
    /// Erases the path/gesture the user drew on the user interface
    fn erase_path(&mut self) {
        let context = self.widgets._draw_handler.get_context().unwrap();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint().unwrap();
        info!("Path of gesture was erased");
    }

    #[cfg(feature = "gesture")]
    /// Paint the path/gesture the user drew with her finger
    fn draw_path(&mut self) {
        // Delete the previous path
        self.erase_path();
        // Set path colors
        let context = self.widgets._draw_handler.get_context().unwrap();
        context.set_operator(cairo::Operator::Over);
        context.set_source_rgba(
            path_defaults::PATHCOLOR.0,
            path_defaults::PATHCOLOR.1,
            path_defaults::PATHCOLOR.2,
            path_defaults::PATHCOLOR.3,
        );
        // Sets the maximum age of a dot to be drawn. This prevents the path from getting to long and obstructing the UI
        let max_age = std::time::Duration::from_millis(path_defaults::PATHFADINGDURATION);
        // Get the newest dots and connect them with a line
        for dot in self
            .model
            .gesture
            .get_swipe_path()
            .iter()
            .rev()
            .take(path_defaults::PATHLENGTH)
        {
            // Check if the dot is fresh enough to get painted
            if dot.time.elapsed() < max_age {
                // Create a line between the previous dot and the current one
                context.line_to(dot.x, dot.y);
            } else {
                break;
            }
        }
        context.set_line_width(path_defaults::PATHWIDTH);
        // Paint the line of dots
        context.stroke().unwrap();
        info!("Path of gesture was drawn");
    }

    #[cfg(feature = "suggestions")]
    fn update_suggestions(&mut self, suggestions: Vec<String>) {
        if let Some(left) = suggestions.get(0) {
            self.widgets.suggestions.left.set_label(&left);
        } else {
            self.widgets.suggestions.left.set_label("");
        }
        if let Some(center) = suggestions.get(1) {
            self.widgets.suggestions.center.set_label(&center);
        } else {
            self.widgets.suggestions.center.set_label("");
        }
        if let Some(right) = suggestions.get(2) {
            self.widgets.suggestions.right.set_label(&right);
        } else {
            self.widgets.suggestions.right.set_label("");
        }
    }
}
