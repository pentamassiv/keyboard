use super::submitter::*;
use crate::user_interface::MessagePipe;
mod ui_sub_connector;
pub use ui_sub_connector::{EmitUIMsg, UIMsg, UISubmitterConnector};

pub const ICON_FOLDER: &str = "./data/icons/";
pub const RESOLUTIONX: i32 = 10000;
pub const RESOLUTIONY: i32 = 10000;

pub const KEYBOARD_DEFAULT_LAYOUT: &str = "us";
pub const KEYBOARD_DEFAULT_VIEW: &str = "base";
use std::collections::HashMap;

mod meta;
mod view;
use view::View;
mod key;
pub use self::meta::*;
use key::Key;

//#[derive(Debug)]
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    submitter: Submitter<ui_sub_connector::UISubmitterConnector>,
}

impl Keyboard {
    pub fn from(
        ui_message_pipe: MessagePipe,
        layout_meta_hashmap: &HashMap<String, LayoutMeta>,
    ) -> Keyboard {
        let ui_sub_connector = ui_sub_connector::UISubmitterConnector::new(ui_message_pipe);
        let submitter = Submitter::new(ui_sub_connector);
        let mut views = HashMap::new();
        for (layout_name, layout_meta) in layout_meta_hashmap {
            for (view_name, key_arrangement) in &layout_meta.views {
                let view = View::from(&key_arrangement, &layout_meta.keys);
                views.insert((layout_name.clone(), view_name.clone()), view);
            }
        }
        let active_view = Keyboard::get_default_layout_view();
        views.shrink_to_fit();
        Keyboard {
            views,
            active_view,
            submitter,
        }
    }

    fn get_default_layout_view() -> (String, String) {
        ("us".to_string(), "base".to_string())
    }

    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    pub fn submit(&mut self, submission: Submission) {
        println!("Submit: {:?}", submission);
        self.submitter.submit(submission);
    }

    pub fn get_closest_key(
        &self,
        layout_name: &str,
        view_name: &str,
        x: i32,
        y: i32,
    ) -> Option<&Key> {
        if let Some(view) = self
            .views
            .get(&(layout_name.to_string(), view_name.to_string()))
        {
            view.get_closest_key(x, y)
        } else {
            None
        }
    }
}
