pub use super::submitter::KeyMotion;
extern crate pretty_env_logger;

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
impl std::fmt::Display for Interaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Interaction::Tap(TapDuration::Short, TapMotion::Press) => write!(f, "ShortPress"),
            Interaction::Tap(TapDuration::Short, TapMotion::Release) => write!(f, "ShortRelease"),
            Interaction::Tap(TapDuration::Long, TapMotion::Press) => write!(f, "LongPress"),
            Interaction::Tap(TapDuration::Long, TapMotion::Release) => write!(f, "LongRelease"),
            Interaction::Swipe(SwipeAction::Begin) => write!(f, "SwipeBegin"),
            Interaction::Swipe(SwipeAction::Update) => write!(f, "SwipeUpdate"),
            Interaction::Swipe(SwipeAction::Finish) => write!(f, "SwipeFinish"),
        }
    }
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
                info!(
                    "Keyboard struct added new view: (layout: {}, view: {})",
                    layout_name, view_name
                );
            }
        }
        views.shrink_to_fit();
        let active_view = Keyboard::get_start_layout_view(layout_names);
        info!(
            "Keyboard starts in layout: {}, view: {}",
            active_view.0, active_view.1
        );
        let interpreter = Interpreter::new();
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
        info!("Keyboard handles {} at x: {}, y: {}", interaction, x, y);
        // Saves the closest key to interaction as active_key
        // This is necessary to ensure a release always releases the last activated button because small moves of the input don't necessaryly trigger a SwipeUpdate
        // This means a user could press a button at its edge and move it just enough for a different button to be returned as closest after slightly moving the finger
        // Now the pressed button would never be released

        let key = if let Interaction::Tap(TapDuration::Short, TapMotion::Press) = interaction {
            self.active_key = self
                .views
                .get(active_view)
                .unwrap()
                .get_closest_key(x, y)
                .cloned();
            info!("Keyboard looked up closest key");
            &self.active_key
        } else {
            info!("Keyboard did not look up the closest key, but used the previously pressed key");
            &self.active_key
        };

        if let Some(key) = key {
            let key = key.clone();

            match interaction {
                Interaction::Tap(_, tap_motion) => {
                    self.ui_connection
                        .emit(Msg::ButtonInteraction(key.get_id(), tap_motion));
                    if let Some(key_actions) = key.get_actions(interaction) {
                        self.execute_tap_action(&key.get_id(), key_actions);
                    };
                }

                Interaction::Swipe(swipe_action) => match swipe_action {
                    SwipeAction::Begin => {
                        self.ui_connection.emit(Msg::ReleaseAllButtions);
                        self.submitter.release_all_keys_and_modifiers();
                    }
                    SwipeAction::Update | SwipeAction::Finish => {}
                },
            }
        }
    }

    fn execute_tap_action(&mut self, key_id: &str, actions_vec: &[KeyAction]) {
        info!("Keyboard handles actions for key {}", key_id);
        let mut prev_layout = self.prev_layout.clone();
        let mut prev_view = self.prev_view.clone();
        for action in actions_vec {
            let (ui_message, submission) = self.get_ui_submitter_msg_from_action(key_id, action);

            if let Some(ui_message) = ui_message {
                // If a change of the layout or the view is requested, the ui will not switch back to the previous layout/view
                if let Msg::ChangeUILayoutView(_, _) = ui_message {
                    if let KeyAction::TempSwitchView(_) = action {
                    } else if let KeyAction::TempSwitchLayout(_) = action {
                    } else {
                        prev_layout = None;
                        prev_view = None;
                        self.prev_layout = None;
                        self.prev_view = None;
                    }
                }
                self.ui_connection.emit(ui_message);
            }
            if let Some(submission) = submission {
                let surrounding_text = self.submitter.get_surrounding_text();
                let interpreted_submissions =
                    self.interpreter.interpret(surrounding_text, submission);
                for submission in interpreted_submissions {
                    self.submitter.submit(submission);
                }
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
        info!("The language was determined to be {}", locale_language);
        for layout_name in available_layout_names {
            if locale_language == layout_name {
                info!("A layout for the language was found");
                return (locale_language, start_view);
            }
        }
        warn!(
            "No layout was found for the language. Falling back to layout: {}",
            start_layout
        );
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
                submission = Some(Submission::Keycode(*keycode));
            }
            KeyAction::ToggleKeycode(keycode) => {
                submission = Some(Submission::ToggleKeycode(*keycode));
            }
            KeyAction::EnterString(text) => submission = Some(Submission::Text(text.to_string())),
            KeyAction::Modifier(modifier) => {
                submission = Some(Submission::Modifier(modifier.clone()));
                ui_message = Some(crate::user_interface::Msg::LatchingButtonInteraction(
                    key_id.to_string(),
                ));
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
