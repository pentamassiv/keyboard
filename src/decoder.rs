// Imports from other crates
use std::sync::mpsc;

// Imports from other modules
use crate::keyboard::UIConnector;
use crate::submitter::Submission;
#[cfg(feature = "suggestions")]
use crate::user_interface::Msg;
use input_decoder::InputDecoder;

/// The Decoder attempts to correct errors and guess the submission the user had in mind when clicking the key.
/// Currently it only changes '  ' to '. '
pub struct Decoder {
    ui_connection: UIConnector,
    receiver: mpsc::Receiver<(String, String)>, // Receives the surrounding text
    text_left_of_cursor: String,
    text_right_of_cursor: String,
    prev_submissions: Vec<Submission>,
    input_decoder: InputDecoder,
    previous_words: Vec<String>,
}

impl Decoder {
    /// Create a new Decoder
    pub fn new(ui_connection: UIConnector, receiver: mpsc::Receiver<(String, String)>) -> Decoder {
        let prev_submissions = Vec::new();
        let text_left_of_cursor = "".to_string();
        let text_right_of_cursor = "".to_string();
        let input_decoder = InputDecoder::new("./language_model.bin");
        let previous_words = Vec::new();
        Decoder {
            ui_connection,
            receiver,
            text_left_of_cursor,
            text_right_of_cursor,
            prev_submissions,
            input_decoder,
            previous_words,
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
        if text_to_decode == " " {
            if self.prev_submissions.last() == Some(&Submission::Text(" ".to_string())) {
                info!("End of sentence suspected because space was entered twice in a row. Will be replaced with '. '");
                new_submissions.push(Submission::Erase(1));
                new_submissions.push(Submission::Text(". ".to_string()));
            } else {
                let no_new_words = self.update_last_words();
                for idx in 0..no_new_words {
                    println!("Entered '{}' into decoder", &self.previous_words[idx]);
                    self.input_decoder.entered_word(&self.previous_words[idx]);
                }

                // Notify the UI about new suggestions
                // Suggestions are not implemented yet so these don't make any sense
                #[cfg(feature = "suggestions")]
                {
                    let predictions = self.input_decoder.get_predictions();
                    let predictions: Vec<String> = predictions
                        .into_iter()
                        .map(|(word, _)| word)
                        .take(3)
                        .collect();

                    self.ui_connection.emit(Msg::Suggestions(predictions));
                }
                new_submissions.push(Submission::Text(text_to_decode));
            }
        } else {
            new_submissions.push(Submission::Text(text_to_decode));
        }
        self.prev_submissions = new_submissions.clone();

        new_submissions
    }

    // Updates the previous_words field and returns the number of new words
    fn update_last_words(&mut self) -> usize {
        let previous_words = &self.previous_words;

        let updated_words: Vec<&str> = self
            .text_left_of_cursor
            .split_ascii_whitespace()
            .rev()
            .take(2)
            .collect();
        let updated_words: Vec<String> = updated_words
            .into_iter()
            .rev()
            .map(|x| x.to_string())
            .collect();

        let no_changed_words = if !previous_words.is_empty()
            && updated_words.get(0) == previous_words.get(previous_words.len() - 1)
        {
            updated_words.len() - 1
        } else {
            println!("Reset language model");
            self.input_decoder.reset();
            updated_words.len()
        };
        self.previous_words = updated_words;
        no_changed_words
    }

    /// Decode an update to a gesture
    /// This is not implemented yet
    pub fn decode_gesture(&mut self, _x: i32, _y: i32) {
        self.update_surrounding_text();
        #[cfg(feature = "suggestions")]
        self.ui_connection
            .emit(Msg::Suggestions(vec!["gesture_calculating".to_string()]));
    }

    /// Notify the decoder about the end of a gesture and get the most likely word
    pub fn get_gesture_result(&mut self, _x: i32, _y: i32) -> String {
        self.update_surrounding_text();
        #[cfg(feature = "suggestions")]
        {
            self.ui_connection.emit(Msg::Suggestions(Vec::new()));
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
