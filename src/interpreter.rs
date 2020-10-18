// Imports from other modules
use crate::submitter::Submission;

/// The Interpreter attempts to correct errors and guess the submission the user had in mind when clicking the key.
/// Currently it only changes '  ' to '. '
pub struct Interpreter {
    prev_submissions: Vec<Submission>,
}

impl Interpreter {
    /// Create a new Interpreter
    pub fn new() -> Interpreter {
        let prev_submissions = Vec::new();
        Interpreter { prev_submissions }
    }

    /// Interprets the submission a key would send while considering the surrounding text and previous submissions.
    /// It returns a vector of the submissions the user had intended
    pub fn interpret(
        &mut self,
        surrounding_text: (String, String),
        submission: Submission,
    ) -> Vec<Submission> {
        info!("Received the surrounding text:");
        info!("Left of the cursor: {}", surrounding_text.0);
        info!("Right of the cursor: {}", surrounding_text.1);
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
        new_submissions
    }
}
