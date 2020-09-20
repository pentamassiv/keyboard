//use super::super::keyboard::EmitUIMsg;
use super::{EmitUIMsg, UIMsg};
use crate::keyboard::vk_ui_connector::UIConnector;
use input_method_service::*;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

pub struct SubConnector {
    ui_connector: UIConnector,
}

impl SubConnector {
    pub fn new(ui_connector: UIConnector) -> SubConnector {
        SubConnector { ui_connector }
    }
    //Removed for testing
    pub fn emit(&self, message: UIMsg) {
        self.ui_connector.message_pipe.emit(message);
    }
}

impl KeyboardVisibility for SubConnector {
    fn show_keyboard(&self) {
        self.ui_connector.message_pipe.emit(UIMsg::Visible(true));
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        self.ui_connector.message_pipe.emit(UIMsg::Visible(false));
        println!("Hide keyboard");
    }
}

impl HintPurpose for SubConnector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.ui_connector
            .message_pipe
            .emit(UIMsg::HintPurpose(content_hint, content_purpose));
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}
