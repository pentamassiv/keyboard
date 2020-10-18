// Imports from other crates
extern crate pretty_env_logger;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

// Imports from other modules
use crate::config::fallback_layout::{FALLBACK_LAYOUT_NAME, FALLBACK_VIEW_NAME};
use crate::interpreter::Interpreter;
pub use crate::submitter::KeyMotion;
use crate::submitter::{Submission, Submitter};
use crate::user_interface::Msg;

// Modules
mod content_connector;
mod key;
mod meta;
mod ui_connector;
mod view;
use key::Key;
use view::View;

// Re-exports
pub use self::meta::{
    KeyAction, KeyArrangement, KeyDisplay, KeyMeta, LayoutMeta, Location, Modifier,
};
pub use ui_connector::UIConnector;

pub const RESOLUTIONX: i32 = 1000;
pub const RESOLUTIONY: i32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Enum to differentiate a press from a release of a key
pub enum TapMotion {
    Press = 1,
    Release = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The UI sends 'Interaction's to the keyboard.
/// These interactions can be a tap or a swipe and potentially cause a key press/release or a word to be entered from recoginzing the gesture
pub enum Interaction {
    /// A tap is when the user touches the UI or removes the finger from the UI. The finger is not moved around on the UI
    Tap(TapDuration, TapMotion),
    /// A swipe is when the user touches the UI and moves her finger around
    Swipe(SwipeAction),
}
impl std::fmt::Display for Interaction {
    /// Makes the Interaction nicely printable
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
/// Duration of a tap
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

/// The keyboard struct is the "heart" of the application. It is the connector between the Interpreter and the Submitter.
/// It also stores the available views, which contain the keys. It is the keyboards job to find out which key the user wanted to press,
/// interpret the keypress and notify the UI and Submitter, if they need to take action. The keyboard also saves which layout/view it was set to before,
/// if the change is only until the next interaction
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    active_key: Option<Key>,
    latched_keys: HashSet<String>,
    prev_layout: Option<String>,
    prev_view: Option<String>,
    ui_connection: UIConnector, // Allows sending messages to the UI
    interpreter: Interpreter,
    submitter: Submitter<ui_connector::UIConnector, content_connector::ContentConnector>,
}

impl Keyboard {
    /// Reads the layout infos and builds a keyboard struct from it
    pub fn from(
        ui_connector: ui_connector::UIConnector,
        layout_meta_hashmap: &HashMap<String, LayoutMeta>,
    ) -> Keyboard {
        // Creates a new submitter and moves a clone of the ui_connector to it
        let ui_connection = ui_connector.clone();
        // Create a new channel. This will be used to send changes of the surrounding text to the interpreter
        let (tx, rx) = mpsc::channel();
        // Create a new interpreter that stores the receiver of the channel
        let interpreter = Interpreter::new(rx);
        // Create a new connection to allow the input_method protocol to notify the keyboard about changes to the surrounding text
        let content_connector = content_connector::ContentConnector::new(tx);
        // Create a new Submitter
        let submitter = Submitter::new(ui_connector, content_connector);

        // Create a view for each 'KeyArrangement'
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

        // Select a layout and view to start with
        let active_view = Keyboard::get_start_layout_view(layout_names);
        info!(
            "Keyboard starts in layout: {}, view: {}",
            active_view.0, active_view.1
        );
        Keyboard {
            views,
            active_view,
            active_key: None,
            latched_keys: HashSet::new(),
            prev_layout: None,
            prev_view: None,
            ui_connection,
            interpreter,
            submitter,
        }
    }

    /// This method is used to tell the keyboard about a new user interaction
    /// The keyboard handles everything from the interpretation to the execution of the actions the key initiates.
    pub fn input(&mut self, x: i32, y: i32, interaction: Interaction) {
        let active_view = &self.active_view;
        info!("Keyboard handles {} at x: {}, y: {}", interaction, x, y);

        // If the interaction was a keypress, the closest key to the interaction is returned and saved as the active_key.
        // If the interaction was too far away from a key 'None' is returned.
        // This increases the speed but it is also necessary to ensure a release always releases the last activated button. Small moves of the finger of a user don't necessaryly trigger a SwipeUpdate
        // This means a user could press a button at its edge and move it just enough for a different button to be returned as closest_button after slightly moving the finger.
        // The wrong button would be released and the pressed button would never be released.
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

        // If the interaction was close enough of a key..
        if let Some(key) = key {
            let key = key.clone();
            // ..check the variant of the interaction
            match interaction {
                // ..if it was a tap
                Interaction::Tap(_, tap_motion) => {
                    // ..send a message to the UI to release or press the button of the key

                    let key_id = key.get_id();
                    match tap_motion {
                        TapMotion::Press => {
                            self.ui_connection
                                .emit(Msg::ButtonInteraction(key.get_id(), tap_motion));
                        }
                        TapMotion::Release => {
                            if !self.latched_keys.contains(&key_id) {
                                self.ui_connection
                                    .emit(Msg::ButtonInteraction(key.get_id(), tap_motion));
                            }
                        }
                    }
                    // ..and execute its actions
                    if let Some(key_actions) = key.get_actions(interaction) {
                        self.execute_tap_action(&key.get_id(), key_actions);
                    };
                }

                // ..if the variant was a swipe, check if it is the beginning, update or end of it
                Interaction::Swipe(swipe_action) => match swipe_action {
                    // .. if it is the begin, send a message to the UI to release all buttons
                    // .. and also tell the submitter to release all keys and modifiers
                    SwipeAction::Begin => {
                        for key_id in self.latched_keys.drain() {
                            self.ui_connection
                                .emit(Msg::ButtonInteraction(key_id, TapMotion::Release));
                        }
                        if let Some(active_key) = &self.active_key {
                            self.ui_connection.emit(Msg::ButtonInteraction(
                                active_key.get_id(),
                                TapMotion::Release,
                            ));
                        }
                        self.active_key = None;
                        self.submitter.release_all_keys_and_modifiers();
                    }
                    // NOT IMPLEMENTED YET
                    // Will tell interpreter to update calculations for gesture recognition and then submit the most likely word
                    SwipeAction::Update | SwipeAction::Finish => {}
                },
            }
        }
    }

    /// Give interpreter the actions of the key and execute the returned actions, then switch to the previous layout/view
    fn execute_tap_action(&mut self, key_id: &str, actions_vec: &[KeyAction]) {
        info!("Keyboard handles actions for key {}", key_id);
        // Save the previous layout and view in a variable because one of the actions might overwrite them later
        // At the end of the method, the UI will be told to switch to this layout and view
        let mut prev_layout = self.prev_layout.clone();
        let mut prev_view = self.prev_view.clone();

        // For each action in the vector..
        for action in actions_vec {
            // .. get the submission and the message for the UI that the action causes. Each action causes at least one of them
            let (ui_message, submission) = self.get_ui_submitter_msg_from_action(key_id, action);

            // If a message for the UI is available, it is sent
            // If the UI is instructed to permanently (NOT just temporarily) switch to a new layout, the values of prev_layout and prev_view
            // are set to None to prevent a switch of the layout/view at the end of the method
            if let Some(ui_message) = ui_message {
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

            // If there is a pending submission,..
            if let Some(submission) = submission {
                // .. give the interpreter the submission to interpret
                let interpreted_submissions = self.interpreter.interpret(submission);
                // Submit each of the returned submissions
                for submission in interpreted_submissions {
                    self.submitter.submit(submission);
                }
            }
        }

        // Switch back to the previous layout/view
        self.switch_back_to_prev_view(prev_layout, prev_view);
    }

    /// Selects one of the available layouts or the fallback layout to start the keyboard with
    fn get_start_layout_view(available_layout_names: HashSet<String>) -> (String, String) {
        // If no other layout/view is selecte, start with the fallback layout/view
        let start_layout = FALLBACK_LAYOUT_NAME.to_string();
        let start_view = FALLBACK_VIEW_NAME.to_string();

        let locale_language = crate::get_locale_language();
        info!("The language was determined to be {}", locale_language);
        // If the language set by locale is available, return early from the method to start the keyboard with that layout/view
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
        // start with the fallback layout/view
        (start_layout, start_view)
    }

    /// Switches the layout/view back to the supplied layout/view
    /// This method is meant to be used to switch back to the previous layout/view
    fn switch_back_to_prev_view(&mut self, prev_layout: Option<String>, prev_view: Option<String>) {
        if prev_layout.is_some() || prev_view.is_some() {
            let ui_message = crate::user_interface::Msg::ChangeUILayoutView(prev_layout, prev_view);
            self.ui_connection.emit(ui_message);
            self.prev_layout = None;
            self.prev_view = None;
        };
    }

    /// Fetch the events from the wayland event queue
    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    /// Submit the text
    pub fn submit_text(&mut self, text: String) {
        self.submitter.submit(Submission::Text(text));
    }

    /// Each KeyAction results in a message to the UI and/or a Submission.
    /// This method translates the KeyAction in its UI message and Submission
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
                // If the modifier key id is present in the latched_keys HashMap, remove it
                if self.latched_keys.remove(key_id) {
                    info! {
                        "'{}' key is no longer latched", key_id
                    }
                } else {
                    info! {
                        "'{}' key is now latched", key_id
                    }
                    // Else insert it
                    self.latched_keys.insert(key_id.to_string());
                }
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
