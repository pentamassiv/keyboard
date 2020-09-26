pub use self::wayland::vk_service::KeyMotion;
use input_method_service::*;
use wayland_client::EventQueue;

pub mod wayland;

#[derive(Debug, PartialEq, Clone)]
pub enum Submission {
    Text(String),
    Keycode(String),
    StickyKeycode(String, KeyMotion),
    Erase(u32),
}

pub struct Submitter<T: 'static + KeyboardVisibility + HintPurpose> {
    event_queue: EventQueue,
    im_service: Option<IMService<T>>,
    virtual_keyboard: Option<wayland::vk_service::VKService>,
}

impl<T: 'static + KeyboardVisibility + HintPurpose> Submitter<T> {
    pub fn new(connector: T) -> Submitter<T> {
        let (event_queue, seat, vk_mgr, im_mgr) = wayland::init_wayland(); //let (seat, layer_shell, vk_mgr, im_mgr) = super::init_wayland();
        let mut im_service = None;
        let mut virtual_keyboard = None;
        if let Some(vk_mgr) = vk_mgr {
            virtual_keyboard = Some(wayland::vk_service::VKService::new(&seat, vk_mgr));
        };
        if let Some(im_mgr) = im_mgr {
            im_service = Some(IMService::new(&seat, im_mgr, connector));
        };

        Submitter {
            event_queue,
            im_service,
            virtual_keyboard,
        }
    }
    pub fn fetch_events(&mut self) {
        self.event_queue
            .dispatch_pending(&mut (), |event, _, _| println!("Event: {:?}", event))
            .unwrap();
    }

    pub fn toggle_shift(&mut self) {
        if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
            virtual_keyboard.toggle_shift();
        }
    }

    pub fn submit(&mut self, submission: Submission) {
        match submission {
            Submission::Text(text) => {
                self.submit_text(text);
            }
            Submission::Keycode(keycode) => {
                if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                    virtual_keyboard.submit_keycode(&keycode);
                };
            }
            Submission::StickyKeycode(keycode, key_motion) => {
                if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                    virtual_keyboard.send_key(&keycode, key_motion);
                };
            }
            Submission::Erase(no_char) => self.erase(no_char),
        }
    }

    fn submit_text(&mut self, text: String) {
        let mut success = Err(SubmitError::NotActive);
        if let Some(im) = &mut self.im_service {
            if im.commit_string(text.clone()).is_ok() && im.commit().is_ok() {
                success = Ok(());
            };
        }
        if success.is_err() {
            if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                virtual_keyboard.submit_keycode(&text);
            // there is no result returned so there is no way of knowing if it was sucessful.
            // a success is assumed
            } else {
                println!("Error: No way to submit");
            }
        }
    }

    fn erase(&mut self, no_char: u32) {
        let mut success = Err(SubmitError::NotActive);
        if let Some(im) = &self.im_service {
            if im.delete_surrounding_text(no_char, 0).is_ok() && im.commit().is_ok() {
                success = Ok(());
            };
        }
        if success.is_err() {
            if let Some(virtual_keyboard) = &mut self.virtual_keyboard {
                for i in 0..no_char {
                    virtual_keyboard.submit_keycode("DELETE"); // TODO: Double check if this is the correct str to delete the last letter
                }

            // there is no result returned so there is no way of knowing if it was sucessful.
            // a success is assumed
            } else {
                println!("No way to delete");
            }
        }
    }
}
