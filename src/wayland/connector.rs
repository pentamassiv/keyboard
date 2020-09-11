use super::super::keyboard::EmitUIMsg;
use super::super::keyboard::UIMsg;
use input_method_service::*;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

struct Connector<'a, T: EmitUIMsg> {
    message_pipe: &'a T,
}

impl<'a, T: EmitUIMsg> Connector<'a, T> {
    fn new(message_pipe: &'a T) -> Connector<'a, T> {
        Connector { message_pipe }
    }
}

impl<'a, T: EmitUIMsg> KeyboardVisability for Connector<'a, T> {
    fn show_keyboard(&self) {
        self.emit(UIMsg::Visable(true));
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        self.emit(UIMsg::Visable(false));
        println!("Hide keyboard");
    }
}

impl<'a, T: EmitUIMsg> HintPurpose for Connector<'a, T> {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.emit(UIMsg::HintPurpose(content_hint, content_purpose));
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}
impl<'a, T: EmitUIMsg> EmitUIMsg for Connector<'a, T> {
    fn emit(&self, message: UIMsg) {
        self.message_pipe.emit(message);
    }
}
