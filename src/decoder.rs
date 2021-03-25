// Imports from other crates
use std::sync::mpsc;

// Imports from other modules
use crate::keyboard::UIConnector;
use crate::submitter::Submission;
#[cfg(feature = "suggestions")]
use crate::user_interface::Msg;
use input_decoder::InputDecoder;

/// The Decoder attempts to correct errors and guess the submission the user had in mind when clicking the key.
pub struct Decoder {
    ui_connection: UIConnector,
    receiver: mpsc::Receiver<(String, String)>, // Receives the surrounding text
    text_left_of_cursor: String,
    text_right_of_cursor: String,
    input_decoder: InputDecoder,
    previous_words: Vec<String>,
    drawn_path: Vec<(f64, f64)>,
}

impl Decoder {
    /// Create a new Decoder
    pub fn new(ui_connection: UIConnector, receiver: mpsc::Receiver<(String, String)>) -> Decoder {
        let text_left_of_cursor = "".to_string();
        let text_right_of_cursor = "".to_string();
        let input_decoder = InputDecoder::new(
            "./language_model.bin",
            path_gen::get_default_buttons_centers(),
            10000,
        );
        let previous_words = Vec::new();
        let drawn_path = Vec::new();
        Decoder {
            ui_connection,
            receiver,
            text_left_of_cursor,
            text_right_of_cursor,
            input_decoder,
            previous_words,
            drawn_path,
        }
    }

    /// Decodes the text that would have been sent while considering the surrounding text and previous submissions.
    /// It returns a vector of the submissions it is assumed the user had intended and updates the suggested next words
    pub fn decode_text(&mut self, text_to_decode: String) -> Vec<Submission> {
        self.update_surrounding_text();
        info!("Received the surrounding text:");
        info!("Left of the cursor: {}", self.text_left_of_cursor);
        info!("Right of the cursor: {}", self.text_right_of_cursor);

        let mut new_submissions = Vec::new();
        // If the current and the previous text submission are a SPACE, it is assumed a sentence was terminated and the previous space gets replaced with a dot
        if text_to_decode.ends_with(' ') {
            if text_to_decode == " " && self.text_left_of_cursor.ends_with(' ') {
                info!("End of sentence suspected because space was entered twice in a row. Will be replaced with '. '");
                new_submissions.push(Submission::Erase(1));
                new_submissions.push(Submission::Text(". ".to_string()));
            } else {
                // Notify the input decoder about the changed previous words
                let no_new_words = self.update_last_words();
                for word in self
                    .previous_words
                    .iter()
                    .skip(self.previous_words.len() - no_new_words)
                {
                    info!("Entered '{}' into decoder", word);
                    self.input_decoder.entered_word(&word);
                }

                // Notify the UI about new suggestions
                #[cfg(feature = "suggestions")]
                self.update_suggestions();
                new_submissions.push(Submission::Text(text_to_decode));
            }
        } else {
            new_submissions.push(Submission::Text(text_to_decode));
        }

        new_submissions
    }

    // Updates the previous_words field and returns the number of new words
    fn update_last_words(&mut self) -> usize {
        let previous_words = &self.previous_words;

        info!(
            "previous_words: {:?}, {:?}",
            previous_words.get(0),
            previous_words.get(1)
        );

        info!("text_left_of_cursor: {}", self.text_left_of_cursor);

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

        info!(
            "updated_words: {:?}, {:?}",
            updated_words.get(0),
            updated_words.get(1)
        );

        let no_changed_words = if !previous_words.is_empty()
            && updated_words.get(0) == previous_words.get(previous_words.len() - 1)
        {
            updated_words.len() - 1
        } else {
            info!("Reset language model");
            self.input_decoder.reset();
            updated_words.len()
        };
        self.previous_words = updated_words;
        info!("no_changed_words: {}", no_changed_words);
        no_changed_words
    }

    /// Add the new coordinate of the drawn path
    pub fn update_gesture(&mut self, x: f64, y: f64) {
        self.drawn_path.push((x, y));
    }

    #[cfg(feature = "suggestions")]
    // Notify the UI about new suggestions
    pub fn update_suggestions(&self) {
        {
            let predictions = self.input_decoder.get_predictions();
            let predictions: Vec<String> = predictions.into_iter().take(3).collect();

            self.ui_connection.emit(Msg::Suggestions(predictions));
        }
    }

    /// Notify the decoder about the end of a gesture and get the most likely word
    pub fn get_gesture_result(&mut self, x: f64, y: f64) -> String {
        // Add the last point to the path
        self.drawn_path.push((x, y));

        // Find the word with the most similar ideal path to the drawn gesture
        let predictions = self.input_decoder.find_similar_words(&self.drawn_path);
        self.drawn_path.clear();

        info!("predictions and their similarity:");
        for (word, _) in predictions.iter().take(10) {
            info!("{}", word);
        }

        let predictions: Vec<String> = predictions
            .into_iter()
            .take(3)
            .map(|(word, _)| word)
            .collect();

        // Send the most likely candidates to the ui to suggest and return the most likely candidate
        let most_likely_word = predictions[0].clone();
        #[cfg(feature = "suggestions")]
        self.ui_connection.emit(Msg::Suggestions(predictions));
        most_likely_word
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
