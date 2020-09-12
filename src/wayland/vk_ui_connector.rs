use super::super::keyboard::EmitUIMsg;

pub struct UIConnector<T: 'static>
where
    T: EmitUIMsg,
{
    pub message_pipe: T,
}

impl<T: 'static> UIConnector<T>
where
    T: EmitUIMsg,
{
    pub fn new(message_pipe: T) -> UIConnector<T> {
        UIConnector { message_pipe }
    }
}
