use super::relm_widget::GridBuilder;
use super::{Msg, Orientation};
use crate::dbus::DBusService;
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
    pub current_layout_view: (String, String),
    prev_layout: String,
}

impl UIManager {
    pub fn new(
        sender: Sender<super::Msg>,
        window: Window,
        stack: Stack,
        current_layout_view: (String, String),
    ) -> UIManager {
        let dbus_service = DBusService::new(sender.clone());
        let prev_layout = current_layout_view.0.clone();
        UIManager {
            sender,
            window,
            stack,
            dbus_service,
            current_layout_view,
            prev_layout,
        }
    }

    pub fn haptic_feedback(&self, is_press: bool) {
        let event;
        if is_press {
            event = "button-pressed".to_string();
        } else {
            event = "button-released".to_string();
        };
        self.dbus_service.haptic_feedback(event);
    }

    pub fn change_visibility(&mut self, new_visibility: bool) {
        if new_visibility {
            self.window.show();
        } else {
            self.window.hide();
        }
        self.dbus_service.change_visibility(new_visibility);
    }

    pub fn change_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        info!(
            "UI_manager tries to change the content hint/purpose to ContentHint: {:?}, ContentPurpose: {:?}. This is not implemented yet.",
            content_hint, content_purpose
        )
    }

    pub fn change_orientation(&mut self, orientation: Orientation) {
        match orientation {
            Orientation::Landscape => {
                let (layout, _) = &self.current_layout_view;
                if layout.ends_with("_wide") {
                    info!("Already in landscape orientation")
                } else {
                    let mut landscape_layout = layout.to_string();
                    landscape_layout.push_str("_wide");
                    match self.change_layout_view(Some(landscape_layout), None) {
                        Ok(()) => info!("Sucessfully changed to landscape orientation"),
                        _ => warn!("Failed to change to landscape orientation"),
                    }
                }
            }
            Orientation::Portrait => {
                let (layout, _) = self.current_layout_view.clone();
                if let Some(portrait_layout) = layout.strip_suffix("_wide") {
                    // If str ends with suffix, Some(prefix) is returned, if not None is returned
                    match self.change_layout_view(Some(portrait_layout.to_string()), None) {
                        // View is changed back to base when orientation is changed
                        Ok(()) => info!("Sucessfully changed to portrait orientation"),
                        _ => warn!("Failed to change to portrait orientation"),
                    }
                } else {
                    info!("Already in portrait orientation")
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
        if let Some(new_layout) = &new_layout {
            if new_layout == "previous" {
                layout = self.prev_layout.clone();
            } else {
                layout = new_layout.to_string();
            }
            view = "base".to_string(); // If the layout is changed, the view is always changed to base because the new layout might not have the same view}
        } else {
            layout = self.current_layout_view.0.clone();
        }
        if let Some(new_view) = new_view {
            view = new_view;
        }
        let new_layout_view_name = GridBuilder::make_grid_name(&layout, &view);
        if self
            .stack
            .get_child_by_name(&new_layout_view_name)
            .is_some()
        {
            self.stack.set_visible_child_name(&new_layout_view_name);
            self.sender
                .send(Msg::ChangeKBLayoutView(layout.clone(), view.clone()))
                .expect("send message");
            if new_layout.is_some() {
                self.prev_layout = self.current_layout_view.0.clone();
            }
            self.current_layout_view = (layout, view);
            info!(
                "UI_manager successfully changed to new layout/view: {}",
                new_layout_view_name
            );
            Ok(())
        } else {
            // Is only a warning because the ui_manager always tries to find a landscape layout. If none is provided, this is not an error but expected to fail
            warn!(
                "UI_manager failed to change to new layout/view because no child with the name {} exist in the gtk::Stack",
                new_layout_view_name
            );
            Err(UIError::LayoutViewNonExistent)
        }
    }
}

pub enum UIError {
    LayoutViewNonExistent,
}
