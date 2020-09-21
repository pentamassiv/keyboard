//use super::super::keyboard::EmitUIMsg;

//use crate::keyboard::vk_ui_connector::UIConnector;
use crate::user_interface::MessagePipe;
use input_method_service::*;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

pub enum UIMsg {
    Visible(bool),
    HintPurpose(ContentHint, ContentPurpose),
    //ChangeUILayoutView(Option<String>, Option<String>),
}

pub trait EmitUIMsg {
    fn emit(&self, message: UIMsg);
}

pub struct UISubmitterConnector {
    pub message_pipe: MessagePipe,
}

impl UISubmitterConnector {
    pub fn new(message_pipe: MessagePipe) -> UISubmitterConnector {
        UISubmitterConnector { message_pipe }
    }
}

impl KeyboardVisibility for UISubmitterConnector {
    fn show_keyboard(&self) {
        self.message_pipe.emit(UIMsg::Visible(true));
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        self.message_pipe.emit(UIMsg::Visible(false));
        println!("Hide keyboard");
    }
}

impl HintPurpose for UISubmitterConnector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.message_pipe
            .emit(UIMsg::HintPurpose(content_hint, content_purpose));
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}
