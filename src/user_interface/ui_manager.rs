// Imports from other crates
use gtk::prelude::{StackExt, WidgetExt};
use gtk::{Stack, Window};
use relm::Sender;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

// Imports from other modules
use super::relm_widget::GridBuilder;
use super::{Msg, Orientation};
use crate::dbus::DBusService;

/// The UIManager changes the layout/view, hides/shows the keyboard and can tell the DBusService to send a button-pressed/button-releases event to give haptic feedback.
/// It handles all changes to the UI except for gesture paths getting displayed
pub struct UIManager {
    sender: Sender<Msg>,
    window: Window,
    stack: Stack,
    dbus_service: DBusService,
    pub current_layout_view: (String, String),
    prev_layout: String,
}

impl UIManager {
    /// Creates a new UIManager
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

    /// Sends a 'button-pressed' event to the DBusService if 'is_press' is true and 'button-released' if it is false
    /// This causes the device to give haptic feedback
    pub fn haptic_feedback(&self, is_press: bool) {
        let event;
        if is_press {
            event = "button-pressed".to_string();
        } else {
            event = "button-released".to_string();
        };
        self.dbus_service.haptic_feedback(event);
    }

    /// Handles the request to change the visibility
    /// If the new visibility is 'true' then the keyboard is shown, if it is 'false' it gets hidden
    pub fn change_visibility(&mut self, new_visibility: bool) {
        if new_visibility {
            self.window.show();
        } else {
            self.window.hide();
        }
        // Notify the DBusService about the change
        self.dbus_service.change_visibility(new_visibility);
    }

    /// Handles a change of the content hint and content purpose
    /// CURRENTLY NOT IMPLEMENTED AND DOES NOTHING
    pub fn change_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        info!(
            "UI_manager tries to change the content hint/purpose to ContentHint: {:?}, ContentPurpose: {:?}. This is not implemented yet.",
            content_hint, content_purpose
        )
    }

    /// Handles a change of the orientation
    /// When the new orientation is 'Landscape', it attempts to change to a layout with the same name plus the suffix '_wide'.
    /// If there is no such layout it does nothing
    /// When the new orientation is 'Portrait', it attempts to change to a layout with the same name but without the suffix '_wide'.
    /// If there is no such layout it does nothing
    pub fn change_orientation(&mut self, orientation: Orientation) {
        match orientation {
            Orientation::Landscape => {
                // Get the name of the current layout/view
                let (layout, _) = &self.current_layout_view;
                // If it already ends with the suffix '_wide', nothing gets changed
                if layout.ends_with("_wide") {
                    info!("Already in landscape orientation")
                } else {
                    let mut landscape_layout = layout.to_string();
                    landscape_layout.push_str("_wide");
                    // Attempts to change to a layout with the same name plus the suffix '_wide'.
                    if let Ok(()) = self.change_layout_view(&Some(landscape_layout), None) {
                        info!("Sucessfully changed to landscape orientation")
                    } else {
                        warn!("Failed to change to landscape orientation")
                    }
                }
            }
            Orientation::Portrait => {
                // Gets the current layout/view name
                let (layout, _) = self.current_layout_view.clone();
                // If it had the suffix '_wide' then 'Some(prefix)' is returned and the pattern matches, if not 'None' is returned and nothing gets changed
                if let Some(portrait_layout) = layout.strip_suffix("_wide") {
                    if let Ok(()) =
                        self.change_layout_view(&Some(portrait_layout.to_string()), None)
                    {
                        // View is changed back to base when orientation is changed
                        info!("Sucessfully changed to portrait orientation")
                    } else {
                        warn!("Failed to change to portrait orientation")
                    }
                } else {
                    info!("Already in portrait orientation")
                }
            }
        }
    }

    /// Attempts to change the layout/view
    /// This fails if the requested layout/view is not available
    /// This not necessarily is an error because this also happens if the keyboard tries to change to an orientation the user did not add a specified layout for
    pub fn change_layout_view(
        &mut self,
        new_layout: &Option<String>,
        new_view: Option<String>,
    ) -> Result<(), UIError> {
        // Get the names of the layout and view to change to
        let (layout, view) = self.make_new_layout_view_name(new_layout, new_view);
        // Get the name the grid would be called
        let new_layout_view_name = GridBuilder::make_grid_name(&layout, &view);
        // If such a grid exists...
        if self
            .stack
            .get_child_by_name(&new_layout_view_name)
            .is_some()
        {
            // Change to it
            self.stack.set_visible_child_name(&new_layout_view_name);
            // Notify the keyboard struct about the change
            self.sender
                .send(Msg::ChangeKBLayoutView(layout.clone(), view.clone()))
                .expect("send message");
            // If not only the view was changed, set the value of 'previous_layout' to the new layout name
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
            // It is only a warning because the ui_manager always tries to find a landscape layout. If none is provided, this is not an error but expected to fail
            warn!(
                "UI_manager failed to change to new layout/view because no child with the name {} exist in the gtk::Stack",
                new_layout_view_name
            );
            Err(UIError::LayoutViewNonExistent)
        }
    }

    /// Returns the layout and view name to change to.
    /// If a new layout name was provided, the method returns the new layout name and 'base' for view.
    /// The layout name 'previous' is special. In that case the name of the previous layout and 'base' for view is returned.
    /// If only a new view was provided, the method returns the name of the current layout and the name of the new view.
    fn make_new_layout_view_name(
        &self,
        new_layout: &Option<String>,
        new_view: Option<String>,
    ) -> (String, String) {
        let layout;
        // Get the current view
        let mut view = self.current_layout_view.1.clone();
        // If the layout is supposed to get changed,
        if let Some(new_layout) = &new_layout {
            // and the new layouts name is 'previous'
            if new_layout == "previous" {
                // return the name of the prevous layout
                layout = self.prev_layout.clone();
            } else {
                // if it was any other layout name, return it
                layout = new_layout.to_string();
            }
            view = "base".to_string(); // If the layout is changed, the view is always changed to base because the new layout might not have the same view}
        } else {
            // If no new layout is requested, return the current layout
            layout = self.current_layout_view.0.clone();
        }
        // If a new view is requested, return the name of the current layout and the name of the new view
        if let Some(new_view) = new_view {
            view = new_view;
        }
        (layout, view)
    }
}

/// Errors when changing the UI
pub enum UIError {
    /// The requested layout was not available
    LayoutViewNonExistent,
}
