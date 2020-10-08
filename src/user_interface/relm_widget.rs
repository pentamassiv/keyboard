use super::gesture_handler::GestureSignal;
use super::{Gestures, Msg, Orientation, UIManager, Widgets, Win};
use crate::config::directories;
use crate::config::input_settings;
use crate::keyboard;
use crate::submitter::*;
use gtk::OverlayExt;
use gtk::*;
use keyboard::UIConnector;
use relm::Channel;
#[cfg(feature = "suggestions")]
use std::collections::HashMap;

mod grid_builder;
pub use grid_builder::GridBuilder;

pub const WINDOW_DEFAULT_HEIGHT: i32 = 720;

impl relm::Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    // Create the widgets.
    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        load_css();

        let message_pipe = UIConnector::new(relm.clone());
        let layout_meta = keyboard::LayoutMeta::new();
        let keyboard = keyboard::Keyboard::from(message_pipe, &layout_meta);
        let (stack, key_refs) = GridBuilder::make_stack(relm, layout_meta);

        let drawing_area = gtk::DrawingArea::new();
        let mut draw_handler = relm::DrawHandler::new().expect("draw handler");
        draw_handler.init(&drawing_area);
        let overlay = gtk::Overlay::new();
        overlay.add(&stack);
        overlay.add_overlay(&drawing_area);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);

        #[cfg(feature = "suggestions")]
        {
            let hbox = make_pref_hbox(relm, &keyboard);
            vbox.add(&hbox);
            info! {"Suggestion buttons added"};
        }
        vbox.add(&overlay);

        let window = Window::new(WindowType::Toplevel);
        window.set_property_default_height(WINDOW_DEFAULT_HEIGHT);
        window.add(&vbox);

        let long_press_gesture = GestureLongPress::new(&drawing_area);
        long_press_gesture.set_property_delay_factor(input_settings::LONG_PRESS_DELAY_FACTOR);
        let drag_gesture = GestureDrag::new(&drawing_area);

        let stream = relm.stream().clone();
        // Create a channel to be able to send a message from another thread.
        let (channel, sender) = Channel::new(move |msg| {
            // This closure is executed whenever a message is received from the sender.
            // We send a message to the current widget.
            stream.emit(msg);
        });

        let (layout_name, view_name) = keyboard.active_view.clone();

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
                overlay,
                draw_handler,
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

    fn init_view(&mut self) {
        if crate::submitter::wayland::get_layer_shell().is_some() {
            let window_clone = self.widgets.window.clone();
            wayland::layer_shell::make_overlay_layer(window_clone);
        }

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

        relm::connect!(
            self.gestures.long_press_gesture,
            connect_pressed(_, x, y), // Long press detected
            self.relm,
            Msg::GestureSignal(x, y, GestureSignal::LongPress)
        );

        relm::connect!(
            self.gestures.drag_gesture,
            connect_drag_begin(_, x, y),
            self.relm,
            Msg::GestureSignal(x, y, GestureSignal::DragBegin)
        );
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
            return (Some(Msg::Quit), Inhibit(false))
        );

        #[cfg(feature = "gesture")]
        relm::connect!(
            // TODO: Is this even necessary since it is drawn every few milliseconds anyways????
            self.relm,
            self.widgets.overlay,
            connect_draw(_, _),
            return (Some(Msg::UpdateDrawBuffer), gtk::Inhibit(false))
        );

        self.widgets.window.show_all(); // All widgets are visible
        self.widgets.window.hide(); // Keyboard starts out being invisible and is only shown if requested via DBus or input-method

        let (layout_name, view_name) = self.keyboard.active_view.clone(); // Set visible child MUST be called after show_all. Otherwise it takes no effect!
        let starting_layout_view = GridBuilder::make_grid_name(&layout_name, &view_name);
        self.widgets
            .stack
            .set_visible_child_name(&starting_layout_view);
        info!("UI layout/view started in {}", starting_layout_view);
        info!("UI initialized");
    }
}

fn load_css() {
    info! {"Trying to load CSS file to customize the keyboard"};
    let provider = gtk::CssProvider::new();
    // Gets PathBuf and tries to convert it to a String
    let css_path_abs = match directories::get_absolute_path(directories::CSS_FILE_REL) {
        Some(path) => path.into_os_string().into_string(),
        None => {
            error! {"Unable to load CSS file because the home directory was not found"};
            return;
        }
    };
    // If conversion was unsuccessfull, return
    let css_path_abs = match css_path_abs {
        Ok(path) => path,
        Err(_) => {
            error! {"Unable to load CSS file because the filepath was not UTF-8 encoded"};
            return;
        }
    };
    match provider.load_from_path(&css_path_abs) {
        Ok(_) => {
            // We give the CssProvided to the default screen so the CSS rules we added
            // can be applied to our window.
            gtk::StyleContext::add_provider_for_screen(
                &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            info! {"CSS file successfully loaded"};
        }
        Err(_) => {
            warn! {"Unable to load CSS file. The file might be missing or broken. Using default CSS"}
        }
    }
}

#[cfg(feature = "suggestions")]
fn make_pref_hbox(relm: &relm::Relm<super::Win>, keyboard: &keyboard::Keyboard) -> Box {
    let suggestion_buttons = make_suggestion_buttons(relm);
    let mut layout_names = Vec::new();
    for (layout_name, _) in keyboard.views.keys() {
        layout_names.push(layout_name);
    }
    let preferences_button = make_pref_button(relm, layout_names);
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    hbox.set_margin_start(0);
    hbox.set_margin_end(0);
    for suggestion_button in suggestion_buttons {
        hbox.add(&suggestion_button);
    }
    hbox.add(&preferences_button);
    hbox
}

#[cfg(feature = "suggestions")]
fn make_pref_button(relm: &relm::Relm<super::Win>, layout_names: Vec<&String>) -> Button {
    let preferences_button = gtk::Button::new();
    preferences_button
        .get_style_context()
        .add_class("preferences");
    preferences_button.set_label("pref");
    preferences_button.set_hexpand(true);
    preferences_button.set_focus_on_click(false);

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
    for unique_layout_name in tmp_layouts.keys() {
        let new_layout_button = gtk::Button::new();
        new_layout_button.set_label(unique_layout_name);
        pref_vbox.add(&new_layout_button);
        let tmp_popover_ref = pref_popover.clone();
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
fn make_suggestion_buttons(relm: &relm::Relm<super::Win>) -> Vec<Button> {
    let mut buttons = Vec::new();
    let button_names = [
        "sug_l".to_string(),
        "sug_c".to_string(),
        "sug_r".to_string(),
    ];
    for name in button_names.iter() {
        let new_suggestion_button = gtk::Button::new();
        new_suggestion_button
            .get_style_context()
            .add_class("suggestions");
        new_suggestion_button.set_label(name);
        new_suggestion_button.set_hexpand(true);
        new_suggestion_button.set_focus_on_click(false);

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
