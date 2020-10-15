pub use self::wayland::vk_service::KeyMotion;
use crate::keyboard;
use std::sync::{Arc, Mutex};
use wayland_client::EventQueue;
use zwp_input_method_service::*;

pub mod wayland;

#[derive(Debug, PartialEq, Clone)]
pub enum Submission {
    Text(String),
    Keycode(u32),
    ToggleKeycode(u32),
    Modifier(keyboard::Modifier),
    Erase(u32),
}

pub struct Submitter<T: 'static + KeyboardVisibility + HintPurpose> {
    event_queue: EventQueue,
    im_service: Option<IMService<T>>,
    virtual_keyboard: Option<Arc<Mutex<wayland::vk_service::VKService>>>,
}

impl<T: 'static + KeyboardVisibility + HintPurpose> Submitter<T> {
    pub fn new(connector: T) -> Submitter<T> {
        let (event_queue, seat, vk_mgr, im_mgr) = wayland::init_wayland();
        let mut im_service = None;
        let mut virtual_keyboard = None;
        if let Some(vk_mgr) = vk_mgr {
            virtual_keyboard = Some(wayland::vk_service::VKService::new(&seat, vk_mgr));
            info!("VirtualKeyboard service available");
        };
        if let Some(im_mgr) = im_mgr {
            im_service = Some(IMService::new(&seat, im_mgr, connector));
            info!("InputMethod service available");
        };

        Submitter {
            event_queue,
            im_service,
            virtual_keyboard,
        }
    }

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

    pub fn get_surrounding_text(&self) -> String {
        if let Some(im) = &self.im_service {
            return im.get_surrounding_text();
        } else {
            warn!("The surrounding text can not be requested because the imput_method protocol is unavailable");
        }
        "".to_string()
    }

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

    pub fn submit(&mut self, submission: Submission) {
        match submission {
            Submission::Text(text) => {
                self.submit_text(text);
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

    fn submit_text(&mut self, text: String) {
        info!("Submitter is trying to submit the text: {}", text);
        let mut success = false;
        if let Some(im) = &mut self.im_service {
            if im.commit_string(text.clone()).is_ok() && im.commit().is_ok() {
                success = true;
            };
        }
        if !success {
            if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                // The virtual_keyboard protocol is very limited regarding text input and can only input individual keys. Trying to submit each character individually
                if virtual_keyboard
                    .lock()
                    .unwrap()
                    .send_unicode_str(&text)
                    .is_ok()
                {
                    success = true;
                }
            }
        }
        if !success {
            error!("Failed to submit the text: {}", text);
        }
    }

    fn erase(&mut self, no_char: u32) {
        info!(
            "Submitter is trying to erase the last {} characters",
            no_char
        );
        let mut success = false;
        if let Some(im) = &self.im_service {
            if im.delete_surrounding_text(no_char, 0).is_ok() && im.commit().is_ok() {
                success = true;
            };
            info!("Submitter successfully used input_method to erase the characters");
        }
        if !success {
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
                        info!(
                            "Submitter successfully used virtual_keyboard to erase the characters"
                        );
                        success = true;
                    }
                }
            }
        }
        if !success {
            error!("Submitter failed to erase the characters");
        }
    }
}
