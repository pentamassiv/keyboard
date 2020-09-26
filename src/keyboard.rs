pub use super::submitter::KeyMotion;
use super::submitter::*;
use crate::interpreter::Interpreter;
use crate::user_interface::Msg;
use std::collections::HashMap;

mod ui_connector;
pub use ui_connector::{EmitUIMsg, UIConnector, UIMsg};
pub mod input_handler;
pub use input_handler::*;
mod meta;
mod view;
use view::View;
mod key;
pub use self::meta::*;
use key::Key;

pub const ICON_FOLDER: &str = "./data/icons/";
pub const RESOLUTIONX: i32 = 1000; // TODO: Think about the exact value
pub const RESOLUTIONY: i32 = 1000;

pub const KEYBOARD_DEFAULT_LAYOUT: &str = "us";
pub const KEYBOARD_DEFAULT_VIEW: &str = "base";

//#[derive(Debug)]
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    input_handler: InputHandler,
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
        for (layout_name, layout_meta) in layout_meta_hashmap {
            for (view_name, key_arrangement) in &layout_meta.views {
                let view = View::from(&key_arrangement, &layout_meta.keys);
                views.insert((layout_name.clone(), view_name.clone()), view);
            }
        }
        let active_view = Keyboard::get_default_layout_view();
        let input_handler = InputHandler::new();
        let interpreter = Interpreter::new();
        views.shrink_to_fit();
        Keyboard {
            views,
            active_view,
            input_handler,
            ui_connection,
            interpreter,
            submitter,
        }
    }

    pub fn input(&mut self, x: i32, y: i32, input: InputType) {
        let active_view = &self.active_view;
        let input_type = self.input_handler.input(input);
        println!("key_action: {:?}", input_type);
        if let Some(key) = self.views.get(active_view).unwrap().get_closest_key(x, y) {
            let key = key.clone();
            match input_type {
                PressType::ShortPress => self
                    .ui_connection
                    .emit(Msg::ButtonInteraction(key.id, KeyMotion::Press)),
                PressType::ShortPressRelease => {
                    self.ui_connection
                        .emit(Msg::ButtonInteraction(key.id.clone(), KeyMotion::Release));
                    if let Some(key_actions) = key.get_actions(&KeyEvent::ShortPress) {
                        self.interpret_execute_action(&key.id, key_actions, input_type);
                    }
                }
                PressType::LongPress => {
                    if let Some(key_actions) = key.get_actions(&KeyEvent::LongPress) {
                        self.interpret_execute_action(&key.id, key_actions, input_type);
                    }
                }
                PressType::LongPressRelease => {
                    self.ui_connection
                        .emit(Msg::ButtonInteraction(key.id.clone(), KeyMotion::Release));
                    if let Some(key_actions) = key.get_actions(&KeyEvent::LongPressRelease) {
                        self.interpret_execute_action(&key.id, key_actions, input_type);
                    };
                }
                PressType::Swipe => self
                    .ui_connection
                    .emit(Msg::ButtonInteraction(key.id, KeyMotion::Release)),
                PressType::SwipeRelease => {}
            }
        }
    }

    fn interpret_execute_action(
        &mut self,
        key_id: &str,
        actions_vec: &[KeyAction],
        press_type: PressType,
    ) {
        for action in actions_vec {
            let mut submission = None;
            let mut ui_message = None;
            match action {
                KeyAction::EnterKeycode(keycode) => match press_type {
                    PressType::ShortPressRelease => {
                        submission = Some(Submission::Keycode(keycode.to_string()));
                    }
                    PressType::LongPress => {
                        submission = Some(Submission::StickyKeycode(
                            keycode.to_string(),
                            KeyMotion::Press,
                        ));
                    }
                    PressType::LongPressRelease => {
                        submission = Some(Submission::StickyKeycode(
                            keycode.to_string(),
                            KeyMotion::Release,
                        ));
                    }
                    _ => println!("Should never be reached"),
                },
                KeyAction::EnterString(text) => {
                    submission = Some(Submission::Text(text.to_string()))
                }
                KeyAction::SwitchView(new_view) => {
                    ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                        None,
                        Some(new_view.to_string()),
                    ));
                }
                KeyAction::SwitchLayout(new_layout) => {
                    ui_message = Some(crate::user_interface::Msg::ChangeUILayoutView(
                        Some(new_layout.to_string()),
                        None,
                    ));
                }
                KeyAction::Modifier(modifier) => {
                    submission = Some(
                        Submission::Keycode("SHIFT".to_string()), // TODO: set up properly
                    );
                }
                KeyAction::Erase => {
                    submission = Some(Submission::Erase(1));
                }
                KeyAction::OpenPopup => {
                    ui_message = Some(crate::user_interface::Msg::OpenPopup(key_id.to_string()));
                }
            }

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
    }

    fn get_default_layout_view() -> (String, String) {
        ("us".to_string(), "base".to_string())
    }

    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    pub fn submit_text(&mut self, text: String) {
        self.submitter.submit(Submission::Text(text));
    }
}
