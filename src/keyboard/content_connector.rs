// Imports from other crates
use std::sync::mpsc;
use zwp_input_method_service::ReceiveSurroundingText;

pub struct ContentConnector {
    pub transmitter: mpsc::Sender<(String, String)>,
}

impl ContentConnector {
    /// Creates a new ContentConnector
    pub fn new(transmitter: mpsc::Sender<(String, String)>) -> ContentConnector {
        ContentConnector { transmitter }
    }
}

/// Implements the ReceiveSurroundingText trait from the zwp_input_method_service crate to notify about changes to the surrounding text
impl ReceiveSurroundingText for ContentConnector {
    fn text_changed(&self, string_left_of_cursor: String, string_right_of_cursor: String) {
        self.transmitter
            .send((string_left_of_cursor, string_right_of_cursor))
            .unwrap();
    }
}
