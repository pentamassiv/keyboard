use super::super::keyboard::EmitUIMsg;
use super::super::keyboard::UIMsg;
use super::vk_ui_connector::UIConnector;
use input_method_service::*;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

pub struct SubConnector<T: 'static>
where
    T: EmitUIMsg,
{
    ui_connector: UIConnector<T>,
}

impl<T: 'static> SubConnector<T>
where
    T: EmitUIMsg,
{
    pub fn new(ui_connector: UIConnector<T>) -> SubConnector<T> {
        SubConnector { ui_connector }
    }
    pub fn emit(&self, message: UIMsg) {
        self.ui_connector.message_pipe.emit(message);
    }
}

impl<T: 'static> KeyboardVisability for SubConnector<T>
where
    T: EmitUIMsg,
{
    fn show_keyboard(&self) {
        self.ui_connector.message_pipe.emit(UIMsg::Visable(true));
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        self.ui_connector.message_pipe.emit(UIMsg::Visable(false));
        println!("Hide keyboard");
    }
}

impl<T: 'static> HintPurpose for SubConnector<T>
where
    T: EmitUIMsg,
{
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        self.ui_connector
            .message_pipe
            .emit(UIMsg::HintPurpose(content_hint, content_purpose));
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}
