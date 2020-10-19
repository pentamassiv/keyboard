// Imports from other crates
use std::sync::mpsc;

// Imports from other modules
use crate::keyboard::UIConnector;
use crate::submitter::Submission;
#[cfg(feature = "suggestions")]
use crate::user_interface::Msg;

/// The Interpreter attempts to correct errors and guess the submission the user had in mind when clicking the key.
/// Currently it only changes '  ' to '. '
pub struct Interpreter {
    ui_connection: UIConnector,
    receiver: mpsc::Receiver<(String, String)>,
    text_left_of_cursor: String,
    text_right_of_cursor: String,
    prev_submissions: Vec<Submission>,
}

impl Interpreter {
    /// Create a new Interpreter
    pub fn new(
        ui_connection: UIConnector,
        receiver: mpsc::Receiver<(String, String)>,
    ) -> Interpreter {
        let prev_submissions = Vec::new();
        let text_left_of_cursor = "".to_string();
        let text_right_of_cursor = "".to_string();
        Interpreter {
            ui_connection,
            receiver,
            text_left_of_cursor,
            text_right_of_cursor,
            prev_submissions,
        }
    }

    /// Interprets the submission a key would send while considering the surrounding text and previous submissions.
    /// It returns a vector of the submissions the user had intended
    pub fn interpret(&mut self, submission: Submission) -> Vec<Submission> {
        self.update_surrounding_text();
        info!("Received the surrounding text:");
        info!("Left of the cursor: {}", self.text_left_of_cursor);
        info!("Right of the cursor: {}", self.text_right_of_cursor);
        let mut new_submissions = Vec::new();
        // Only text submissions are interpreted so far. All other submissions are not altered
        match submission {
            Submission::Text(current_submission) => {
                // If the current and the previous text submission are a SPACE, it is assumed a sentence was terminated and the previous space gets replaced with a dot
                if current_submission == " "
                    && self.prev_submissions.last() == Some(&Submission::Text(" ".to_string()))
                {
                    info!("End of sentence suspected because space was entered twice in a row. Will be replaced with '. '");
                    new_submissions.push(Submission::Erase(1));
                    new_submissions.push(Submission::Text(". ".to_string()));
                } else {
                    new_submissions.push(Submission::Text(current_submission));
                }
            }
            Submission::Erase(_)
            | Submission::Keycode(_)
            | Submission::ToggleKeycode(_)
            | Submission::Modifier(_) => {
                new_submissions.push(submission);
            }
        }
        self.prev_submissions = new_submissions.clone();

        // Notify the UI about new suggestions
        // Suggestions are not implemented yet so these don't make any sense
        #[cfg(feature = "suggestions")]
        self.ui_connection.emit(Msg::Suggestions((
            Some("A".to_string()),
            None,
            Some("sug_right".to_string()),
        )));
        new_submissions
    }

    fn update_surrounding_text(&mut self) {
        // Initalitze the variable
        let mut text_changed = None;
        // Try to get updates of the surrounding string from the receiver
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
                        error!("The channel to keep the interpreter updated of the surrounding text was closed");
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
