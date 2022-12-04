// Imports from other crates
use std::collections::HashMap;

// Imports from other modules
use crate::keyboard::{Interaction, KeyAction, KeyMeta, TapDuration, TapMotion};

#[derive(Debug, Clone)]
/// A key stores all actions that will be executed when the key is pressed as well as its id
pub struct Key {
    id: String,
    actions: HashMap<Interaction, Vec<KeyAction>>,
}

impl Key {
    // Create the key_name key from the provided KeyMeta
    pub fn from(key_name: &str, key_meta: &KeyMeta) -> Key {
        let mut actions = key_meta.actions.clone();
        Self::add_feedback_actions(&mut actions);
        Key {
            id: key_name.to_string(),
            actions,
        }
    }

    /// Returns the id of the Key
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Returns the actions that will be executed when the key is pressed
    pub fn get_actions(&self, interaction: Interaction) -> Option<&Vec<KeyAction>> {
        self.actions.get(&interaction)
    }

    /// Add the action to give feedback when a button is pressed or released to all vectors of KeyActions
    fn add_feedback_actions(actions: &mut HashMap<Interaction, Vec<KeyAction>>) {
        // Create a vector of all variants of TapDuration and TapMotion to get all combinations in the following for loop
        let durations = vec![TapDuration::Short, TapDuration::Long];
        let mut tap_motions = Vec::new();
        tap_motions.push(TapMotion::Press);
        tap_motions.push(TapMotion::Release);

        // Add the action for each of the combinations
        for duration in durations {
            for tap_motion in &tap_motions {
                let interaction = Interaction::Tap(duration, *tap_motion);
                let actions_vec = actions.get_mut(&interaction);
                if let Some(actions_vec) = actions_vec {
                    actions_vec.push(KeyAction::FeedbackPressRelease(
                        *tap_motion == TapMotion::Press,
                    ));
                } else {
                    let actions_vec = vec![KeyAction::FeedbackPressRelease(
                        *tap_motion == TapMotion::Press,
                    )];
                    actions.insert(interaction, actions_vec);
                }
            }
        }
    }
}
