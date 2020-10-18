// Imports from other crates
use std::collections::HashMap;

// Imports from other modules
use crate::keyboard::{Interaction, KeyAction, KeyMeta};

#[derive(Debug, Clone)]
/// A key stores all actions that will be executed when the key is pressed as well as its id
pub struct Key {
    id: String,
    actions: HashMap<Interaction, Vec<KeyAction>>,
}

impl Key {
    // Create the key_name key from the provided KeyMeta
    pub fn from(key_name: &str, key_meta: &KeyMeta) -> Key {
        Key {
            id: key_name.to_string(),
            actions: key_meta.actions.clone(),
        }
    }

    // Returns the id of the Key
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    // Returns the actions that will be executed when the key is pressed
    pub fn get_actions(&self, interaction: Interaction) -> Option<&Vec<KeyAction>> {
        self.actions.get(&interaction)
    }
}
