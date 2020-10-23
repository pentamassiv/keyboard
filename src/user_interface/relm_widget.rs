// Imports from other crates
use gtk::{
    ContainerExt, CssProviderExt, GestureDragExt, GtkWindowExt, OverlayExt, StackExt, WidgetExt,
};
use relm::Channel;

#[cfg(feature = "suggestions")]
use gtk::{ButtonExt, StyleContextExt};
#[cfg(feature = "suggestions")]
use std::collections::HashMap;

// Imports from other modules
use super::gesture_handler::GestureSignal;
#[cfg(feature = "suggestions")]
use super::Suggestions;
use super::{Gestures, Msg, Orientation, UIManager, Widgets, Win};
use crate::config::directories;
use crate::config::input_settings;
use crate::submitter::wayland;
use crate::{keyboard, keyboard::UIConnector};

// Modules
mod grid_builder;

// Re-exports
pub use grid_builder::GridBuilder;

pub const WINDOW_DEFAULT_HEIGHT: i32 = 720;

/// Used to build the UI
impl relm::Widget for Win {
    // Specify the type of the root widget.
    type Root = gtk::Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    // Create the widgets.
    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        // Load a CSS stylesheet to customize the looks of the keyboard
        load_css();

        // Make a connector to allow messages being sent to the UI
        // This will be used by both the keyboard and the Submitter
        let message_pipe = UIConnector::new(relm.clone());
        // Get the meta data needed to build the keyboard
        let layout_meta = keyboard::LayoutMeta::deserialize();
        // Build the keyboard struct that stores all logic of the keys
        let keyboard = keyboard::Keyboard::from(message_pipe, &layout_meta);
        // Build the stack of grids of the layouts from the meta data
        let (stack, key_refs) = GridBuilder::make_stack(relm, layout_meta);
        // Make a new drawing area on which the gesture paths will get painted to
        let drawing_area = gtk::DrawingArea::new();
        let mut draw_handler = relm::DrawHandler::new().expect("draw handler");
        draw_handler.init(&drawing_area);
        // Overlay the drawing area over the stack of layouts
        let overlay = gtk::Overlay::new();
        overlay.add(&stack);
        overlay.add_overlay(&drawing_area);

        // Make the vertical box that stores the overlay and the box of suggestions
        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 2);

        #[cfg(feature = "suggestions")]
        let suggestions;
        #[cfg(feature = "suggestions")]
        {
            // Make the box of suggestions
            let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            h_box.set_margin_start(0);
            h_box.set_margin_end(0);
            let suggestions_and_pref_buttons = make_suggestions_and_pref_buttons(relm, &keyboard);
            suggestions = Suggestions {
                left: suggestions_and_pref_buttons.get(0).unwrap().clone(),
                center: suggestions_and_pref_buttons.get(1).unwrap().clone(),
                right: suggestions_and_pref_buttons.get(2).unwrap().clone(),
            };
            for button in suggestions_and_pref_buttons {
                h_box.add(&button);
            }
            v_box.add(&h_box);
            info! {"Suggestion buttons added"};
        }
        v_box.add(&overlay);

        // Make the window that contains the UI
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_property_default_height(WINDOW_DEFAULT_HEIGHT);
        window.add(&v_box);

        // Add a GestureLongPress handler to the drawing area
        let long_press_gesture = gtk::GestureLongPress::new(&drawing_area);
        long_press_gesture.set_property_delay_factor(input_settings::LONG_PRESS_DELAY_FACTOR);
        let drag_gesture = gtk::GestureDrag::new(&drawing_area);

        // Create a channel to be able to send a message from another thread.
        let stream = relm.stream().clone();
        let (channel, sender) = Channel::new(move |msg| {
            // This closure is executed whenever a message is received from the sender.
            // We send a message to the UI
            stream.emit(msg);
        });

        // Get the name of the currently active layout/view of the keyboard struct
        let (layout_name, view_name) = keyboard.active_view.clone();

        // Make the UIManager that handles e.g. the changing of the layout/view
        let ui_manager = UIManager::new(
            sender,
            window.clone(),
            stack.clone(),
            (layout_name, view_name),
        );
        Win {
            relm: relm.clone(),
            model,
            keyboard,
            key_refs,
            widgets: Widgets {
                window,
                _overlay: overlay,
                _draw_handler: draw_handler,
                #[cfg(feature = "suggestions")]
                suggestions,
                stack,
            },
            gestures: Gestures {
                long_press_gesture,
                drag_gesture,
            },
            ui_manager,
            _channel: channel,
        }
    }

    /// Initialize the view
    /// This includes adding callbacks for GTK events and starting the UI with the currently active layout/view
    fn init_view(&mut self) {
        // Try making the window a layer
        if wayland::get_layer_shell().is_some() {
            wayland::layer_shell::make_overlay_layer(&self.widgets.window);
        }

        // Send a message with the new orientation to the UI whenever the orientation gets changed
        let relm_clone = self.relm.clone(); // Is moved in closure
        self.widgets
            .window
            .connect_configure_event(move |_, configure_event| {
                let (width, _) = configure_event.get_size();
                let orientation = if width == 720 {
                    Orientation::Landscape
                } else {
                    Orientation::Portrait
                };
                relm_clone
                    .stream()
                    .emit(Msg::ChangeUIOrientation(orientation));
                false
            });

        // Send a 'GestureSignal' message to the UI with the coordinates and a GestureSignal::LongPress variant when there was a long press on the overlay
        relm::connect!(
            self.gestures.long_press_gesture,
            connect_pressed(_, x, y), // Long press detected
            self.relm,
            Msg::GestureSignal(x, y, GestureSignal::LongPress)
        );

        // Send a 'GestureSignal' message to the UI with the coordinates and a GestureSignal::DragBegin variant when the beginning of a drag was detected on the overlay
        relm::connect!(
            self.gestures.drag_gesture,
            connect_drag_begin(_, x, y),
            self.relm,
            Msg::GestureSignal(x, y, GestureSignal::DragBegin)
        );

        // Send a 'GestureSignal' message to the UI with the coordinates and a GestureSignal::DragUpdate variant when a drag was already detected
        // on the overlay and the finger was moved was
        relm::connect!(
            self.gestures.drag_gesture,
            connect_drag_update(drag_gesture, x_offset, y_offset),
            self.relm,
            {
                let (x_start, y_start) =
                    drag_gesture.get_start_point().unwrap_or((-1000.0, -1000.0)); // When popup is opened, there is no startpoint. To avoid being close to any buttons this large negative number is given
                let x = x_start + x_offset;
                let y = y_start + y_offset;
                Msg::GestureSignal(x, y, GestureSignal::DragUpdate)
            }
        );

        // Send a 'GestureSignal' message to the UI with the coordinates and a GestureSignal::DragEnd variant when a drag was already detected
        // on the overlay and the finger was lifted off the screen
        relm::connect!(
            self.gestures.drag_gesture,
            connect_drag_end(drag_gesture, x_offset, y_offset),
            self.relm,
            {
                let (x_start, y_start) =
                    drag_gesture.get_start_point().unwrap_or((-1000.0, -1000.0)); // When popup is opened, there is no startpoint. To avoid being close to any buttons this large negative number is given
                let x = x_start + x_offset;
                let y = y_start + y_offset;
                Msg::GestureSignal(x, y, GestureSignal::DragEnd)
            }
        );

        // Connect the signal `delete_event` to send the `Quit` message.
        relm::connect!(
            self.relm,
            self.widgets.window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), gtk::Inhibit(false))
        );

        // Send a 'UpdateDrawBuffer' message to the UI
        #[cfg(feature = "gesture")]
        relm::connect!(
            self.relm,
            self.widgets._overlay,
            connect_draw(_, _),
            return (Some(Msg::UpdateDrawBuffer), gtk::Inhibit(false))
        );

        self.widgets.window.show_all(); // All widgets are visible
        self.widgets.window.hide(); // Keyboard starts out being invisible and is only shown if requested via DBus or input-method

        // Set the visible grid to the currently active layout/view to start with
        let (layout_name, view_name) = self.keyboard.active_view.clone(); // Set visible child MUST be called after show_all. Otherwise it takes no effect!
        let starting_layout_view = GridBuilder::make_grid_name(&layout_name, &view_name);
        self.widgets
            .stack
            .set_visible_child_name(&starting_layout_view);
        info!("UI layout/view started in {}", starting_layout_view);
        info!("UI initialized");
    }
}

/// Loads a CSS stylesheet from a default path to customize the looks of the keyboard
fn load_css() {
    info! {"Trying to load CSS file to customize the keyboard"};
    let provider = gtk::CssProvider::new();
    // Gets PathBuf and tries to convert it to a String
    let css_path_abs = if let Some(path) = directories::get_absolute_path(directories::CSS_FILE_REL)
    {
        path.into_os_string().into_string()
    } else {
        error! {"Unable to load CSS file because the home directory was not found"};
        return;
    };

    // If conversion was unsuccessfull, return
    let css_path_abs = if let Ok(path) = css_path_abs {
        path
    } else {
        error! {"Unable to load CSS file because the filepath was not UTF-8 encoded"};
        return;
    };

    // Try to load the stylesheet
    match provider.load_from_path(&css_path_abs) {
        Ok(_) => {
            // Give the CssProvided to the default screen so the CSS rules from the stylesheet
            // can be applied to the window.
            gtk::StyleContext::add_provider_for_screen(
                &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            info! {"CSS file successfully loaded"};
        }
        Err(_) => {
            warn! {"Unable to load CSS file from path '{}'. The file might be missing or broken. Using default CSS",css_path_abs}
        }
    }
}

#[cfg(feature = "suggestions")]
/// Create the suggestion buttons and a button to open the preferences
fn make_suggestions_and_pref_buttons(
    relm: &relm::Relm<super::Win>,
    keyboard: &keyboard::Keyboard,
) -> Vec<gtk::Button> {
    // Make the buttons to display suggestions
    let mut buttons = make_suggestion_buttons(relm);
    // Get the names of all layouts
    let layout_names: Vec<&String> = keyboard
        .get_views()
        .keys()
        .map(|(layout_name, _)| layout_name)
        .collect();
    // Make a button that openes the preferences
    let preferences_button = make_pref_button(relm, layout_names);
    // Add the preferences button
    buttons.push(preferences_button);
    buttons
}

#[cfg(feature = "suggestions")]
/// Make a preferences button
/// Currently it only allows switching between the layouts
fn make_pref_button(relm: &relm::Relm<super::Win>, layout_names: Vec<&String>) -> gtk::Button {
    // Make the button
    let preferences_button = gtk::Button::new();
    preferences_button
        .get_style_context()
        .add_class("preferences");
    preferences_button.set_label("pref");
    preferences_button.set_hexpand(true);
    preferences_button.set_focus_on_click(false);

    // Add a popover to the button to select one of the available layouts
    let pref_popover = gtk::Popover::new(Some(&preferences_button));
    let pref_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    pref_popover.add(&pref_vbox);
    let mut tmp_layouts = HashMap::new();
    for layout_name in layout_names {
        // Only layouts that are for portrait mode can be switched to.
        //Layouts for landscape mode are switched automatically to when the orientation changes
        if layout_name.strip_suffix("_wide").is_none() {
            tmp_layouts.insert(layout_name, ());
        }
    }
    // For each of the available layouts
    for unique_layout_name in tmp_layouts.keys() {
        // Make a new button labeled with the layout name
        let new_layout_button = gtk::Button::new();
        new_layout_button.set_label(unique_layout_name);
        pref_vbox.add(&new_layout_button);
        let tmp_popover_ref = pref_popover.clone();
        // When the button is clicked, a request to the UI is sent to switch to that layout
        new_layout_button.connect_clicked(move |_| tmp_popover_ref.hide());
        relm::connect!(
            relm,
            new_layout_button,
            connect_button_release_event(clicked_button, _),
            return (
                Some(crate::user_interface::Msg::ChangeUILayoutView(
                    Some(clicked_button.get_label().unwrap().to_string()),
                    None
                )),
                gtk::Inhibit(false)
            )
        );
    }
    preferences_button.connect_clicked(move |_| pref_popover.show_all());
    preferences_button
}

#[cfg(feature = "suggestions")]
/// Makes a suggestion button
fn make_suggestion_buttons(relm: &relm::Relm<super::Win>) -> Vec<gtk::Button> {
    // Make a vector of strings
    // These will be the labels of the suggestion buttons they start with
    let mut buttons = Vec::new();
    let button_names = [
        "sug_l".to_string(),
        "sug_c".to_string(),
        "sug_r".to_string(),
    ];
    // For each of these strings
    for name in button_names.iter() {
        // .. make a new button
        let new_suggestion_button = gtk::Button::new();
        new_suggestion_button
            .get_style_context()
            .add_class("suggestions");
        new_suggestion_button.set_label(name);
        new_suggestion_button.set_hexpand(true);
        new_suggestion_button.set_focus_on_click(false);

        // .. that when clicked will ask the UI to submit its label
        let relm_clone = relm.clone();
        let suggestion_closure = move |button: &gtk::Button| {
            relm_clone
                .stream()
                .emit(Msg::SubmitText(button.get_label().unwrap().to_string()))
        };

        new_suggestion_button.connect_clicked(suggestion_closure);

        buttons.push(new_suggestion_button);
    }

    buttons
}
