// Imports from other crates
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use zwp_input_method_service::{HintPurpose, KeyboardVisibility};

// Imports from other modules
use crate::user_interface::Msg;
use crate::user_interface::Win;

#[derive(Clone)]
/// This is a connection to send messages to the UI
/// It is used by the input_method service to notify the UI about requested changes to the visibility or content hint/purpose
pub struct UIConnector {
    message_pipe: relm::Relm<crate::user_interface::Win>,
}

impl UIConnector {
    /// Creates a new UIConnector
    pub fn new(message_pipe: relm::Relm<Win>) -> UIConnector {
        UIConnector { message_pipe }
    }
    // Send the message to the UI
    pub fn emit(&self, msg: Msg) {
        self.message_pipe.stream().emit(msg)
    }
}

/// Implements the KeyboardVisibility trait from the zwp_input_method_service crate to notify the UI about requested changes to the visibility
impl KeyboardVisibility for UIConnector {
    fn show_keyboard(&self) {
        self.emit(Msg::SetVisibility(true));
        info!("Requested to show the keyboard");
    }
    fn hide_keyboard(&self) {
        self.emit(Msg::SetVisibility(false));
        info!("Requested to hide the keyboard");
    }
}

/// Implements the KeyboardVisibility trait from the zwp_input_method_service crate to notify the UI about requested changes to the content hint/purpose
impl HintPurpose for UIConnector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.emit(Msg::HintPurpose(content_hint, content_purpose));
        info!(
            "Requested to change to ContentHint: {:?} and  ContentPurpose: {:?}",
            content_hint, content_purpose
        );
    }
}
