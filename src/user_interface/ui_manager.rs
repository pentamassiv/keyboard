use super::dbus::DBusService;
use super::{Msg, Orientation};
use gtk::{Stack, StackExt, WidgetExt, Window};
use relm::Sender;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

pub struct UIManager {
    sender: Sender<Msg>,
    window: Window,
    stack: Stack,
    dbus_service: DBusService,
    current_layout_view: (String, String),
}

impl UIManager {
    pub fn new(
        sender: Sender<super::Msg>,
        window: Window,
        stack: Stack,
        current_layout_view: (String, String),
    ) -> UIManager {
        let dbus_service = DBusService::new(sender.clone()).unwrap();
        UIManager {
            sender,
            window,
            stack,
            dbus_service,
            current_layout_view,
        }
    }

    pub fn change_visibility(&mut self, new_visibility: bool) {
        println!("Msg visiblility: {}", new_visibility);
        if new_visibility {
            self.window.show();
        } else {
            self.window.hide();
        }
        self.dbus_service.change_visibility(new_visibility);
    }

    pub fn change_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        println!(
            "ContentHint: {:?}, ContentPurpose: {:?}",
            content_hint, content_purpose
        )
    }

    pub fn change_orientation(&mut self, orientation: Orientation) {
        match orientation {
            Orientation::Landscape => {
                let (layout, _) = &self.current_layout_view;
                if layout.ends_with("_wide") {
                    println!("Already in landscape orientation")
                } else {
                    let mut landscape_layout = layout.to_string();
                    landscape_layout.push_str("_wide");
                    match self.change_layout_view(Some(landscape_layout), None) {
                        Ok(()) => println!("Sucessfully changed to landscape orientation"),
                        _ => println!("Failed to change to landscape orientation"),
                    }
                }
            }
            Orientation::Portrait => {
                let (layout, _) = self.current_layout_view.clone();
                if let Some(portrait_layout) = layout.strip_suffix("_wide") {
                    // If str ends with suffix, Some(prefix) is returned, if not None is returned
                    match self.change_layout_view(Some(portrait_layout.to_string()), None) {
                        // View is changed back to base when orientation is changed
                        Ok(()) => println!("Sucessfully changed to portrait orientation"),
                        _ => println!("Failed to change to portrait orientation"),
                    }
                } else {
                    println!("Already in portrait orientation")
                }
            }
        }
    }

    pub fn change_layout_view(
        &mut self,
        new_layout: Option<String>,
        new_view: Option<String>,
    ) -> Result<(), UIError> {
        let layout;
        let mut view = self.current_layout_view.1.clone();
        if let Some(new_layout) = new_layout {
            layout = new_layout;
            view = "base".to_string(); // If the layout is changed, the view is always changed to base because the new layout might not have the same view
        } else {
            layout = self.current_layout_view.0.clone();
        }
        if let Some(new_view) = new_view {
            view = new_view;
        }
        let new_layout_view_name = crate::keyboard::Keyboard::make_view_name(&layout, &view);
        if self
            .stack
            .get_child_by_name(&new_layout_view_name)
            .is_some()
        {
            self.stack.set_visible_child_name(&new_layout_view_name);
            self.sender
                .send(Msg::ChangeKBLayoutView(layout.clone(), view.clone()))
                .expect("send message");
            self.current_layout_view = (layout, view);
            Ok(())
        } else {
            println!(
                "The requested layout {} does not exist",
                new_layout_view_name
            );
            Err(UIError::LayoutViewError)
        }
    }
}

pub enum UIError {
    LayoutViewError,
}
