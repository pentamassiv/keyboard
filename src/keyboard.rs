// Imports from other crates
extern crate pretty_env_logger;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

// Imports from other modules
use crate::config::fallback_layout::{FALLBACK_LAYOUT_NAME, FALLBACK_VIEW_NAME};
use crate::decoder::Decoder;
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

/// The keyboard struct is the "heart" of the application. It is the connector between the Decoder and the Submitter.
/// It also stores the available views, which contain the keys. It is the keyboards job to find out which key the user wanted to press,
/// Decode the keypress and notify the UI and Submitter, if they need to take action. The keyboard also saves which layout/view it was set to before,
/// if the change is only until the next interaction
pub struct Keyboard {
    views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    latched_keys: HashSet<String>,
    active_key: Option<Key>,
    layout_of_active_key: String, // Necessary to remember to release the key on the correct layout after a switch of the layout
    view_of_active_key: String, // Necessary to remember to release the key on the correct view after a switch of the view
    next_layout: Option<String>,
    next_view: Option<String>,
    ui_connection: UIConnector, // Allows sending messages to the UI
    decoder: Decoder,
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
        // Create a new channel. This will be used to send changes of the surrounding text to the decoder
        let (tx, rx) = mpsc::channel();
        // Create a new decoder that stores the receiver of the channel
        let decoder = Decoder::new(ui_connection.clone(), rx);
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

                if layout_name == "us" && view_name == "base" {
                    println!("US layouts base views:");
                    for ((x, y), key) in &view.key_coordinates {
                        println!("(\"{}\".to_string(), {}, {}),", key.get_id(), x, y);
                    }
                }

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

        let layout_of_active_key = active_view.0.clone();
        let view_of_active_key = active_view.1.clone();

        info!(
            "Keyboard starts in layout: {}, view: {}",
            active_view.0, active_view.1
        );
        Keyboard {
            views,
            active_view,
            latched_keys: HashSet::new(),
            active_key: None,
            layout_of_active_key,
            view_of_active_key,
            next_layout: None,
            next_view: None,
            ui_connection,
            decoder,
            submitter,
        }
    }

    /// Get the views the keyboard has
    pub fn get_views(&self) -> &HashMap<(String, String), View> {
        &self.views
    }

    fn get_idealized_coordinate(&self, x: f64, y: f64) -> (f64, f64) {
        if let Some(active_view) = self.views.get(&self.active_view) {
            (x, active_view.get_row_to_column_ratio() * y)
        } else {
            // This should never happen
            panic!("The name of the active view was not found in the list of available views. This should never happen");
        }
    }

    /// This method is used to tell the keyboard about a new user interaction
    /// The keyboard then handles everything from the decoding to the execution of the actions the key initiates. The submitter and
    /// the UI get notified when they need to take action
    pub fn input(&mut self, x: f64, y: f64, interaction: Interaction) {
        let (x, y) = self.get_idealized_coordinate(x, y);
        info!("Keyboard handles {} at x: {}, y: {}", interaction, x, y);
        // Differentiate between a tap and a swipe
        match interaction {
            Interaction::Tap(_, _) => {
                self.handle_tap(x, y, interaction);
            }
            Interaction::Swipe(swipe_action) => {
                self.handle_swipe(x, y, swipe_action);
            }
        }
    }

    /// Handle an Interaction::Tap
    /// Find out, which key was pressed or released and then execute its actions
    fn handle_tap(&mut self, x: f64, y: f64, interaction: Interaction) {
        let active_view = &self.active_view.clone();
        // If it was a short press, find out which key was pressed
        // Otherwise use the last active key
        // This is an option because if the interaction was too far away from any buttons, it returns 'None'
        let key = if let Interaction::Tap(TapDuration::Short, TapMotion::Press) = interaction {
            self.active_key = self
                .views
                .get(active_view)
                .unwrap()
                .get_closest_key(x, y)
                .cloned();
            info!("Keyboard looked up closest key");
            // Remember which layout and view the key was on to be able to release the correct key later on
            let (layout_of_active_key, view_of_active_key) = self.active_view.clone();
            self.layout_of_active_key = layout_of_active_key;
            self.view_of_active_key = view_of_active_key;
            &self.active_key
        } else {
            info!("Keyboard did not look up the closest key, but used the previously pressed key");
            &self.active_key
        };

        // If the interaction was close enough to a key..
        if let Some(key) = key {
            // .. execute its actions
            let key = key.clone();
            self.execute_tap_actions(&key, interaction);
        }
    }

    /// Handle swipe interactions
    /// If it was the beginning of a swipe, all keys are released
    /// If it was an update, update the calculations for the gesture recognition
    /// If it was the end of a gesture, submit the most likely word
    fn handle_swipe(&mut self, x: f64, y: f64, swipe_action: SwipeAction) {
        match swipe_action {
            // If it is the beginning, ..
            SwipeAction::Begin => {
                let (layout_of_active_key, view_of_active_key) = (
                    self.layout_of_active_key.to_string(),
                    self.view_of_active_key.to_string(),
                );
                // ..send a message to the UI to release all buttons
                for key_id in self.latched_keys.drain() {
                    self.ui_connection.emit(Msg::ButtonInteraction(
                        layout_of_active_key.clone(),
                        view_of_active_key.clone(),
                        key_id,
                        TapMotion::Release,
                    ));
                }
                if let Some(active_key) = &self.active_key {
                    self.ui_connection.emit(Msg::ButtonInteraction(
                        layout_of_active_key,
                        view_of_active_key,
                        active_key.get_id(),
                        TapMotion::Release,
                    ));
                    self.active_key = None;
                }
                // .. and also tell the submitter to release all keys and modifiers
                self.submitter.release_all_keys_and_modifiers();
            }
            // NOT IMPLEMENTED YET
            // Tells decoder to update calculations for gesture recognition
            SwipeAction::Update => self.decoder.update_gesture(x, y),
            // Submits the most likely word
            SwipeAction::Finish => {
                let text = self.decoder.get_gesture_result(x, y);
                self.submit_text(text, true);
            }
        }
    }

    /// Execute the actions that the key causes when it is tapped
    /// EnterString actions get decoded before they get submitted
    fn execute_tap_actions(&mut self, key: &Key, interaction: Interaction) {
        info!("Keyboard handles actions for key {}", key.get_id());

        // Switch back to the previous layout/view
        self.switch_back_to_prev_view();

        if let Some(action_vec) = key.get_actions(interaction) {
            // Execute each action of the vector
            for action in action_vec {
                match action {
                    KeyAction::FeedbackPressRelease(press) => {
                        let (layout_of_active_key, view_of_active_key) = (
                            self.layout_of_active_key.to_string(),
                            self.view_of_active_key.to_string(),
                        );
                        // Pressing a button always notifies the UI about it
                        if *press {
                            self.ui_connection.emit(Msg::ButtonInteraction(
                                layout_of_active_key,
                                view_of_active_key,
                                key.get_id(),
                                TapMotion::Press,
                            ));
                        }
                        // A release only gets sent to the UI if the key is no longer latched
                        else if !self.latched_keys.contains(&key.get_id()) {
                            self.ui_connection.emit(Msg::ButtonInteraction(
                                layout_of_active_key,
                                view_of_active_key,
                                key.get_id(),
                                TapMotion::Release,
                            ));
                        }
                    }
                    KeyAction::EnterKeycode(keycode) => {
                        let submission = Submission::Keycode(*keycode);
                        self.submitter.submit(submission);
                    }
                    KeyAction::ToggleKeycode(keycode) => {
                        let submission = Submission::ToggleKeycode(*keycode);
                        self.submitter.submit(submission);
                    }
                    // Strings get decoded before they are sent
                    KeyAction::EnterString(text) => {
                        let decoded_submissions = self.decoder.decode_text(text.to_string());
                        // Submit each of the returned submissions
                        for submission in decoded_submissions {
                            self.submitter.submit(submission);
                        }
                    }
                    // Modifiers always latch. They are not released when the user lifts of the finger, but when the key is pressed a second time
                    KeyAction::Modifier(modifier) => {
                        let submission = Submission::Modifier(modifier.clone());
                        let key_id = key.get_id();
                        // If the modifier key id is present in the latched_keys HashMap, remove it
                        if self.latched_keys.remove(&key_id) {
                            info! {
                                "'{}' key is no longer latched", key_id
                            }
                        }
                        // Otherwise insert it
                        else {
                            info! {
                                "'{}' key is now latched", key_id
                            }
                            self.latched_keys.insert(key_id.to_string());
                        }
                        self.submitter.submit(submission);
                    }
                    // Delete one char
                    KeyAction::Erase => {
                        let submission = Submission::Erase(1);
                        self.submitter.submit(submission);
                    }
                    KeyAction::SwitchView(new_view) => {
                        self.switch_layout(None, Some(new_view.to_string()), false);
                    }
                    KeyAction::TempSwitchView(new_view) => {
                        self.switch_layout(None, Some(new_view.to_string()), true);
                    }
                    KeyAction::SwitchLayout(new_layout) => {
                        self.switch_layout(Some(new_layout.to_string()), None, false);
                    }
                    KeyAction::TempSwitchLayout(new_layout) => {
                        self.switch_layout(Some(new_layout.to_string()), None, true);
                    }
                    KeyAction::OpenPopup => {
                        let ui_message = Msg::OpenPopup(key.get_id());
                        self.ui_connection.emit(ui_message);
                    }
                }
            }
        }
    }

    /// Tells the UI to switch to a different layout/view and if it is not a permanent switch, it stores the layout/view to switch back to when the next button is pressed
    fn switch_layout(
        &mut self,
        new_layout: Option<String>,
        new_view: Option<String>,
        temporary: bool,
    ) {
        // If it is only temporarily, save the layout/view to switch back to
        if temporary {
            self.next_layout = Some(self.active_view.0.clone());
            self.next_view = Some(self.active_view.1.clone());
        }
        // Notify the UI about the change
        let ui_message = Msg::ChangeUILayoutView(new_layout, new_view);
        self.ui_connection.emit(ui_message);
    }

    /// Switches the layout/view back to the supplied layout/view
    /// This method is meant to be used to switch back to the previous layout/view
    fn switch_back_to_prev_view(&mut self) {
        let prev_layout = &self.next_layout;
        let prev_view = &self.next_view;
        if prev_layout.is_some() || prev_view.is_some() {
            let ui_message = crate::user_interface::Msg::ChangeUILayoutView(
                prev_layout.clone(),
                prev_view.clone(),
            );
            self.ui_connection.emit(ui_message);
            self.next_layout = None;
            self.next_view = None;
        };
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

    /// Fetch the events from the wayland event queue
    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    /// Submit the text
    pub fn submit_text(&mut self, text: String, append_space: bool) {
        self.submitter.submit(Submission::Text(text));
        if append_space {
            let decoded_submissions = self.decoder.decode_text(" ".to_string());
            // Submit each of the returned submissions
            for submission in decoded_submissions {
                self.submitter.submit(submission);
            }
        }
    }
}
