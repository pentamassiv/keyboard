use crate::user_interface::MessagePipe;

pub struct UIConnector {
    pub message_pipe: MessagePipe,
}

impl UIConnector {
    pub fn new(message_pipe: MessagePipe) -> UIConnector {
        UIConnector { message_pipe }
    }
}
