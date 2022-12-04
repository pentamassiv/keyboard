// Imports from other crates
use std::sync::mpsc;

// Imports from other modules
use crate::keyboard::UIConnector;
use crate::submitter::Submission;
#[cfg(feature = "suggestions")]
use crate::user_interface::Msg;

/// The Decoder attempts to correct errors and guess the submission the user had in mind when clicking the key.
/// Currently it only changes '  ' to '. '
pub struct Decoder {
    #[allow(dead_code)]
    ui_connection: UIConnector,
    receiver: mpsc::Receiver<(String, String)>, // Receives the surrounding text
    text_left_of_cursor: String,
    text_right_of_cursor: String,
    prev_submissions: Vec<Submission>,
}

impl Decoder {
    /// Create a new Decoder
    pub fn new(ui_connection: UIConnector, receiver: mpsc::Receiver<(String, String)>) -> Decoder {
        let prev_submissions = Vec::new();
        let text_left_of_cursor = "".to_string();
        let text_right_of_cursor = "".to_string();
        Decoder {
            ui_connection,
            receiver,
            text_left_of_cursor,
            text_right_of_cursor,
            prev_submissions,
        }
    }

    /// Decodes the text that would have been sent while considering the surrounding text and previous submissions.
    /// It returns a vector of the submissions it is assumed the user had intended
    /// Currently it doesn't do much
    /// It only changes '  ' to '. '
    pub fn decode_text(&mut self, text_to_decode: String) -> Vec<Submission> {
        self.update_surrounding_text();
        info!("Received the surrounding text:");
        info!("Left of the cursor: {}", self.text_left_of_cursor);
        info!("Right of the cursor: {}", self.text_right_of_cursor);
        let mut new_submissions = Vec::new();
        // If the current and the previous text submission are a SPACE, it is assumed a sentence was terminated and the previous space gets replaced with a dot
        if text_to_decode == " "
            && self.prev_submissions.last() == Some(&Submission::Text(" ".to_string()))
        {
            info!("End of sentence suspected because space was entered twice in a row. Will be replaced with '. '");
            new_submissions.push(Submission::Erase(1));
            new_submissions.push(Submission::Text(". ".to_string()));
        } else {
            new_submissions.push(Submission::Text(text_to_decode));
        }
        self.prev_submissions = new_submissions.clone();

        // Notify the UI about new suggestions
        // Suggestions are not implemented yet so these don't make any sense
        #[cfg(feature = "suggestions")]
        self.ui_connection.emit(Msg::Suggestions((
            Some("sug_left".to_string()),
            Some("sug_center".to_string()),
            Some("sug_right".to_string()),
        )));
        new_submissions
    }

    /// Decode an update to a gesture
    /// This is not implemented yet
    pub fn decode_gesture(&mut self, _x: i32, _y: i32) {
        self.update_surrounding_text();
        #[cfg(feature = "suggestions")]
        self.ui_connection.emit(Msg::Suggestions((
            None,
            Some("gesture_calculating".to_string()),
            None,
        )));
    }

    /// Notify the decoder about the end of a gesture and get the most likely word
    pub fn get_gesture_result(&mut self, _x: i32, _y: i32) -> String {
        self.update_surrounding_text();
        #[cfg(feature = "suggestions")]
        {
            self.ui_connection
                .emit(Msg::Suggestions((None, None, None)));
            return "gesture".to_string();
        };
        "".to_string()
    }

    /// Updates the decoders knowledge about the surrounding text
    fn update_surrounding_text(&mut self) {
        // Initalitze the variable
        let mut text_changed = None;
        // Try to get updates of the surrounding string from the receiver until there are no updates
        loop {
            match self.receiver.try_recv() {
                // If there was an update, overwrite the surrounding text
                Ok(received_text) => {
                    text_changed = Some(received_text);
                }
                // If no update was available, find out why
                Err(err) => match err {
                    // All updates of the surrounding text have been read
                    mpsc::TryRecvError::Empty => {
                        break;
                    }
                    // The channel was closed
                    mpsc::TryRecvError::Disconnected => {
                        error!("The channel to keep the decoder updated of the surrounding text was closed");
                        panic!()
                    }
                },
            };
        }
        if let Some((left_string, right_string)) = text_changed {
            self.text_left_of_cursor = left_string;
            self.text_right_of_cursor = right_string;
        }
    }
}
