use crate::submitter::Submission;

pub struct Interpreter {
    prev_submissions: Vec<Submission>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let prev_submissions = Vec::new();
        Interpreter { prev_submissions }
    }

    pub fn interpret(
        &mut self,
        surrounding_text: String,
        submission: Submission,
    ) -> Vec<Submission> {
        info!("Received the surrounding text: {}", surrounding_text);
        let mut new_submissions = Vec::new();
        match submission {
            Submission::Text(current_submission) => {
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
