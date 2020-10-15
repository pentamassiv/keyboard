use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use zwp_input_method_service::{HintPurpose, KeyboardVisibility};

use crate::user_interface::Msg;
use crate::user_interface::Win;

pub enum UIMsg {
    Visible(bool),
    HintPurpose(ContentHint, ContentPurpose),
}

pub trait EmitUIMsg {
    fn emit_ui_msg(&self, message: UIMsg);
}

#[derive(Clone)]
pub struct UIConnector {
    pub message_pipe: relm::Relm<crate::user_interface::Win>,
}

impl UIConnector {
    pub fn new(message_pipe: relm::Relm<Win>) -> UIConnector {
        UIConnector { message_pipe }
    }
    pub fn emit(&self, msg: Msg) {
        self.message_pipe.stream().emit(msg)
    }
}

impl KeyboardVisibility for UIConnector {
    fn show_keyboard(&self) {
        self.emit_ui_msg(UIMsg::Visible(true));
        info!("Requested to show the keyboard");
    }
    fn hide_keyboard(&self) {
        self.emit_ui_msg(UIMsg::Visible(false));
        info!("Requested to hide the keyboard");
    }
}

impl HintPurpose for UIConnector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.emit_ui_msg(UIMsg::HintPurpose(content_hint, content_purpose));
        info!(
            "Requested to change to ContentHint: {:?} and  ContentPurpose: {:?}",
            content_hint, content_purpose
        );
    }
}

impl EmitUIMsg for UIConnector {
    fn emit_ui_msg(&self, message: UIMsg) {
        match message {
            UIMsg::Visible(visable) => {
                self.emit(Msg::Visible(visable));
            }
            UIMsg::HintPurpose(content_hint, content_purpose) => {
                self.emit(Msg::HintPurpose(content_hint, content_purpose));
            }
        }
    }
}
