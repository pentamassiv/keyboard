// Imports from other crates
use std::sync::{Arc, Mutex};
use wayland_client::EventQueue;
use zwp_input_method_service::InputMethod;
use zwp_input_method_service::{HintPurpose, IMService, IMVisibility, ReceiveSurroundingText};

// Imports from other modules
pub use self::wayland::vk_service::KeyMotion;
use crate::keyboard;

// Modules
pub mod wayland;

#[derive(Debug, PartialEq, Clone)]
/// Possible types of submissions
pub enum Submission {
    /// Submit a string
    Text(String),
    /// Emulates a physical key that is pressed and released
    Keycode(u32),
    /// Emulates a physical key that gets toggled (if it was previously released, emulates a press and if it was previously pressed, emulates a release)
    ToggleKeycode(u32),
    /// Emulates a modifier getting pressed (e.g SHIFT)
    Modifier(keyboard::Modifier),
    /// Erase the number of chars before the cursor
    Erase(u32),
}

/// Handles all submissions
pub struct Submitter<T: 'static + IMVisibility + HintPurpose, D: 'static + ReceiveSurroundingText> {
    event_queue: EventQueue,
    im_service: Option<IMService<T, D>>,
    virtual_keyboard: Option<Arc<Mutex<wayland::vk_service::VKService>>>,
}

impl<T: IMVisibility + HintPurpose, D: ReceiveSurroundingText> Submitter<T, D> {
    /// Creates a new Submitter
    pub fn new(ui_connector: T, content_connector: D) -> Submitter<T, D> {
        // Gets all necessary wayland objects to use the available protocols
        let (event_queue, seat, vk_mgr, im_mgr) = wayland::init_wayland();
        let mut im_service = None;
        let mut virtual_keyboard = None;
        // Tries to create a VKService (wrapper for the virtual_keyboard protocol). The shell has to support to protocol to be available
        if let Some(vk_mgr) = vk_mgr {
            virtual_keyboard = Some(wayland::vk_service::VKService::new(&seat, &vk_mgr));
            info!("VirtualKeyboard service available");
        };
        // Tries to create a IMService (wrapper for the input_method protocol). The shell has to support to protocol to be available
        if let Some(im_mgr) = im_mgr {
            im_service = Some(IMService::new(
                &seat,
                im_mgr,
                ui_connector,
                content_connector,
            ));
            info!("InputMethod service available");
        };

        Submitter {
            event_queue,
            im_service,
            virtual_keyboard,
        }
    }

    /// Fetch new events from the wayland event queue
    pub fn fetch_events(&mut self) {
        self.event_queue
            .dispatch_pending(&mut (), |event, _, _| {
                error!(
                    "Wayland event received, that was not handled. Event: {:?}",
                    event
                )
            })
            .unwrap();
    }

    /// Sends requests to release all keys and modifiers
    pub fn release_all_keys_and_modifiers(&mut self) {
        if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
            if virtual_keyboard
                .lock()
                .unwrap()
                .release_all_keys_and_modifiers()
                .is_err()
            {
                error!("Submitter failed to release all keys and modifiers");
            }
        }
    }

    /// Submits the Submission with the available protocol.
    /// If available the input_method protocol is tried first because it is simpler to submit strings
    /// and more reliable when it comes to submitting non-ascii chars
    pub fn submit(&mut self, submission: Submission) {
        // Depending on the variant of submission, a different method is executed
        match submission {
            Submission::Text(text) => {
                self.submit_text(&text);
            }
            Submission::Keycode(keycode) => {
                info!("Submitter is trying to submit the keycode: {}", keycode);
                if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                    if virtual_keyboard
                        .lock()
                        .unwrap()
                        .press_release_key(keycode)
                        .is_err()
                    {
                        error!(
                            "Submitter failed to press and release the keycode {}",
                            keycode
                        );
                    }
                } else {
                    error!(
                        "Virtual_keyboard protocol not available! Unable to submit keycode {}",
                        keycode
                    )
                };
            }
            Submission::ToggleKeycode(keycode) => {
                info!("Submitter is trying to toggle the keycode: {}", keycode);
                if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                    if virtual_keyboard
                        .lock()
                        .unwrap()
                        .toggle_key(keycode)
                        .is_err()
                    {
                        error!("Submitter failed to toggle the keycode {}", keycode);
                    }
                } else {
                    error!(
                        "Virtual_keyboard protocol not available! Unable to toggle keycode {}",
                        keycode
                    )
                };
            }
            Submission::Modifier(modifier) => {
                info!(
                    "Submitter is trying to toggle the modifier '{:?}'",
                    modifier
                );
                if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                    if virtual_keyboard
                        .lock()
                        .unwrap()
                        .toggle_modifier(modifier)
                        .is_err()
                    {
                        error!("Submitter failed to toggle the modifier");
                    }
                } else {
                    error!("Virtual_keyboard protocol not available! Unable to toggle modifier")
                };
            }
            Submission::Erase(no_char) => {
                self.erase(no_char);
            }
        }
    }

    /// Try to submit the text
    /// If the input_method protocol is available, use it to submit the string as a whole.
    /// If it is not available, submit each character individually via virtual_keyboard protocol (This is error prone and should only be used as a last resort).
    fn submit_text(&mut self, text: &str) {
        info!("Submitter is trying to submit the text: {}", text);
        if let Some(im) = &mut self.im_service {
            if im.commit_string(text.to_string()).is_ok() && im.commit().is_ok() {
                return;
            };
        }

        if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
            // The virtual_keyboard protocol is very limited regarding text input and can only input individual keys. Trying to submit each character individually
            if virtual_keyboard
                .lock()
                .unwrap()
                .send_unicode_str(&text)
                .is_ok()
            {
                return;
            }
        }

        error!("Failed to submit the text: {}", text);
    }

    /// Erases the specified amount of chars left of the cursor
    /// Uses the input_method protocol if available. As a fallback it sends press/release requests of the DELETE key repeatedly.
    fn erase(&mut self, no_char: u32) {
        info!(
            "Submitter is trying to erase the last {} characters",
            no_char
        );
        if let Some(im) = &self.im_service {
            if im
                .delete_surrounding_text(no_char.try_into().unwrap(), 0)
                .is_ok()
                && im.commit().is_ok()
            {
                info!("Submitter successfully used input_method to erase the characters");
                return;
            };
        }

        if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
            for _ in 0..no_char {
                // Keycode for 'DELETE is 111
                if virtual_keyboard
                    .lock()
                    .unwrap()
                    .press_release_key(111)
                    .is_err()
                {
                    break;
                } else {
                    info!("Submitter successfully used virtual_keyboard to erase the characters");
                }
            }
            return;
        }

        error!("Submitter failed to erase the characters");
    }
}
