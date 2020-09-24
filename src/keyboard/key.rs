use crate::keyboard::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Key {
    pub id: String,
    actions: HashMap<KeyEvent, Vec<KeyAction>>,
}

impl Key {
    pub fn from(key_name: &str, key_meta: &KeyMeta) -> Key {
        Key {
            id: key_name.to_string(),
            actions: key_meta.actions.clone(),
        }
    }
    pub fn activate(&self, win: &crate::user_interface::Win, key_event: &KeyEvent) {
        let tmp_vec = Vec::new();
        let actions_vec = self.actions.get(&key_event).unwrap_or(&tmp_vec);
        for action in actions_vec {
            match action {
                KeyAction::EnterKeycode(keycode) => {
                    win.relm.stream().emit(crate::user_interface::Msg::Submit(
                        Submission::Keycode(keycode.to_string()),
                    ));
                    //self.submitter.submit(Submission::Keycode(keycode))
                }
                KeyAction::EnterString(text) => {
                    win.relm
                        .stream()
                        .emit(crate::user_interface::Msg::Submit(Submission::Text(
                            text.to_string(),
                        )));
                    //self.submitter.submit(Submission::Keycode(keycode))
                }
                KeyAction::SwitchView(new_view) => {
                    let switch_view_msg = crate::user_interface::Msg::ChangeUILayoutView(
                        None,
                        Some(new_view.to_string()),
                    );
                    win.relm.stream().emit(switch_view_msg);
                }
                KeyAction::Modifier(modifier) => {
                    win.relm.stream().emit(crate::user_interface::Msg::Submit(
                        Submission::Keycode("SHIFT".to_string()), // TODO: set up properly
                    ));
                }
                KeyAction::Erase => {
                    win.relm
                        .stream()
                        .emit(crate::user_interface::Msg::Submit(Submission::Erase));
                }
                KeyAction::OpenPopup => {
                    // TODO
                    //self.popover.show_all();
                }
            }
        }
        //self.button.activate(); // Disabled, because the transition takes too long and makes it looks sluggish
    }
}
