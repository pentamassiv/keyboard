pub use super::submitter::KeyMotion;
use super::submitter::*;
use crate::config::fallback_layout::*;
use crate::interpreter::Interpreter;
use crate::user_interface::Msg;
use std::collections::{HashMap, HashSet};

mod ui_connector;
pub use ui_connector::{EmitUIMsg, UIConnector, UIMsg};
mod meta;
mod view;
use view::View;
mod key;
pub use self::meta::*;
use key::Key;

pub const ICON_FOLDER: &str = "./data/icons/";
pub const RESOLUTIONX: i32 = 1000; // TODO: Think about the exact value
pub const RESOLUTIONY: i32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapMotion {
    Press = 1,
    Release = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interaction {
    Tap(TapDuration, TapMotion),
    Swipe(SwipeAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapDuration {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeAction {
    Begin,
    Update,
    Finish,
}

//#[derive(Debug)]
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    active_key: Option<Key>,
    prev_layout: Option<String>,
    prev_view: Option<String>,
    ui_connection: UIConnector,
    interpreter: Interpreter,
    submitter: Submitter<ui_connector::UIConnector>,
}

impl Keyboard {
    pub fn from(
        ui_connector: ui_connector::UIConnector,
        layout_meta_hashmap: &HashMap<String, LayoutMeta>,
    ) -> Keyboard {
        let ui_connection = ui_connector.clone();
        let submitter = Submitter::new(ui_connector);
        let mut views = HashMap::new();
        let mut layout_names = HashSet::new();
        for (layout_name, layout_meta) in layout_meta_hashmap {
            layout_names.insert(layout_name.to_string());
            for (view_name, key_arrangement) in &layout_meta.views {
                let view = View::from(&key_arrangement, &layout_meta.keys);
                views.insert((layout_name.clone(), view_name.clone()), view);
            }
        }
        let active_view = Keyboard::get_start_layout_view(layout_names);
        println!("starting view: {}, {}", active_view.0, active_view.1);
        let interpreter = Interpreter::new();
        views.shrink_to_fit();
        Keyboard {
            views,
            active_view,
            active_key: None,
            prev_layout: None,
            prev_view: None,
            ui_connection,
            interpreter,
            submitter,
        }
    }

    pub fn input(&mut self, x: i32, y: i32, interaction: Interaction) {
        let active_view = &self.active_view;
        println!("key_action: {:?}", interaction);
        // Saves the closest key to interaction as active_key
        // This is necessary to ensure a release always releases the last activated button because small moves of the input don't necessaryly trigger a SwipeUpdate
        // This means a user could press a button at its edge and move it just enough for a different button to be returned as closest after slightly moving the finger
        // Now the pressed button would never be released
        let key = match interaction {
            Interaction::Tap(TapDuration::Short, TapMotion::Press) => {
                self.active_key = self
                    .views
                    .get(active_view)
                    .unwrap()
                    .get_closest_key(x, y)
                    .cloned();
                &self.active_key
            }
            _ => &self.active_key,
        };
        if let Some(key) = key {
            let key = key.clone();

            match interaction {
                Interaction::Tap(_, tap_motion) => {
                    self.ui_connection
                        .emit(Msg::ButtonInteraction(key.id.to_string(), tap_motion));
                    if let Some(key_actions) = key.get_actions(&interaction) {
                        self.execute_tap_action(&key.id, key_actions);
                    };
                }

                Interaction::Swipe(swipe_action) => match swipe_action {
                    SwipeAction::Begin => {
                        self.ui_connection
                            .emit(Msg::ButtonInteraction(key.id, TapMotion::Release));
                        self.submitter.release_all_keys();
                    }
                    SwipeAction::Update => {}
                    SwipeAction::Finish => {}
                },
            }
        }
    }

    fn execute_tap_action(&mut self, key_id: &str, actions_vec: &[KeyAction]) {
        println!(
            "execute_action: key_id {}, actions_vec {:?}",
            key_id, actions_vec
        );
        let prev_layout = self.prev_layout.clone();
        let prev_view = self.prev_view.clone();
        for action in actions_vec {
            let (ui_message, submission) = self.get_ui_submitter_msg_from_action(key_id, action);
            // Each action can only result in eighter a ui_message or a submission
            if let Some(submission) = submission {
                let interpreted_submissions = self.interpreter.interpret(submission);
                for submission in interpreted_submissions {
                    self.submitter.submit(submission);
                }
            } else if let Some(ui_message) = ui_message {
                self.ui_connection.emit(ui_message);
            }
        }

        self.switch_back_to_prev_view(prev_layout, prev_view);
    }

    fn get_start_layout_view(available_layout_names: HashSet<String>) -> (String, String) {
        let start_layout = FALLBACK_LAYOUT_NAME.to_string();
        let start_view = FALLBACK_VIEW_NAME.to_string();
        let locale = format!("{}", locale_config::Locale::user_default());
        let locale_language: String = locale.rsplit('-').take(1).collect();
        let locale_language = locale_language.to_lowercase();
        println!("local language: {}", locale_language);
        for layout_name in available_layout_names {
            if locale_language == layout_name {
                return (locale_language, start_view);
            }
        }
        println!("WARNING: Fallback layout loaded");
        (start_layout, start_view)
    }

    fn switch_back_to_prev_view(&mut self, prev_layout: Option<String>, prev_view: Option<String>) {
        // Switch view because last keypress said to switch back to old view
        if prev_layout.is_some() || prev_view.is_some() {
            let ui_message = crate::user_interface::Msg::ChangeUILayoutView(prev_layout, prev_view);
            self.ui_connection.emit(ui_message);
            self.prev_layout = None;
            self.prev_view = None;
        };
    }

    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    pub fn submit_text(&mut self, text: String) {
        self.submitter.submit(Submission::Text(text));
    }

    fn get_ui_submitter_msg_from_action(
        &mut self,
        key_id: &str,
        action: &KeyAction,
    ) -> (Option<Msg>, Option<Submission>) {
        let mut submission = None;
        let mut ui_message = None;
        match action {
            KeyAction::EnterKeycode(keycode) => {
                submission = Some(Submission::Keycode(keycode.to_string()));
            }
            KeyAction::ToggleKeycode(keycode) => {
                submission = Some(Submission::ToggleKeycode(keycode.to_string()));
            }
            KeyAction::EnterString(text) => submission = Some(Submission::Text(text.to_string())),
            KeyAction::Modifier(modifier) => {
                submission = Some(
                    Submission::Keycode("SHIFT".to_string()), // TODO: set up properly
                );
            }
            KeyAction::Erase => {
                submission = Some(Submission::Erase(1));
            }
            KeyAction::SwitchView(new_view) => {
                ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                    None,
                    Some(new_view.to_string()),
                ));
            }
            KeyAction::TempSwitchView(new_view) => {
                ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                    None,
                    Some(new_view.to_string()),
                ));
                self.prev_view = Some(self.active_view.1.clone());
            }
            KeyAction::SwitchLayout(new_layout) => {
                ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                    Some(new_layout.to_string()),
                    None,
                ));
            }
            KeyAction::TempSwitchLayout(new_layout) => {
                ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                    Some(new_layout.to_string()),
                    None,
                ));
                self.prev_layout = Some(self.active_view.0.clone());
            }
            KeyAction::OpenPopup => {
                ui_message = Some(crate::user_interface::Msg::OpenPopup(key_id.to_string()));
            }
        }
        (ui_message, submission)
    }
}
