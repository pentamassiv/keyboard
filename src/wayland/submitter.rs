use input_method_service::*;

pub enum Submission {
    Text(String),
    Keycode(String),
    Erase,
}

pub struct Submitter<T: 'static + KeyboardVisability + HintPurpose> {
    im_service: Option<IMService<T>>,
    virtual_keyboard: Option<super::vk_service::VKService>,
}

impl<T: 'static + KeyboardVisability + HintPurpose> Submitter<T> {
    fn new(connector: T) -> Submitter<T> {
        let (seat, layer_shell, vk_mgr, im_mgr) = super::init_wayland();
        let im_service = None;
        let virtual_keyboard = None;
        if let Some(vk_mgr) = vk_mgr {
            virtual_keyboard = Some(super::vk_service::VKService::new(&seat, vk_mgr));
        };
        if let Some(im_mgr) = im_mgr {
            let im_service = Some(IMService::new(&seat, im_mgr, connector));
        };

        Submitter {
            im_service,
            virtual_keyboard,
        }
    }

    pub fn submit(&self, submission: Submission) {
        match submission {
            Submission::Text(text) => self.submit_text(text),
            Submission::Keycode(keycode) => self.virtual_keyboard.unwrap().submit_keycode(&keycode),
            Submission::Erase => self.erase(),
        }
    }

    fn submit_text(&self, text: String) {
        if let Some(im) = &self.im_service {
            im.commit_string(text);
            im.commit();
        } else if let Some(virtual_keyboard) = &self.virtual_keyboard {
            virtual_keyboard.submit_keycode(&text);
        } else {
            println!("No way to submit");
        }
    }

    fn erase(&self) {
        // TODO
        let x = 0;
    }
}
