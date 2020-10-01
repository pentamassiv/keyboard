use crate::keyboard::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Key {
    pub id: String,
    actions: HashMap<Interaction, Vec<KeyAction>>,
}

impl Key {
    pub fn from(key_name: &str, key_meta: &KeyMeta) -> Key {
        Key {
            id: key_name.to_string(),
            actions: key_meta.actions.clone(),
        }
    }
    pub fn get_actions(&self, interaction: &Interaction) -> Option<&Vec<KeyAction>> {
        self.actions.get(interaction)
    }
}