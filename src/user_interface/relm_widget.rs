use super::{Gestures, InputType, Msg, Orientation, UIManager, Widgets, Win};
use crate::config::directories;
use crate::keyboard;
use crate::submitter::*;
use gtk::OverlayExt;
use gtk::*;
use keyboard::UIConnector;
use relm::Channel;
use std::time::Instant;

#[cfg(feature = "suggestions")]
use std::collections::HashMap;

mod grid_builder;
pub use grid_builder::GridBuilder;

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

        #[cfg(feature = "suggestions")]
        let suggestion_buttons = make_suggestion_buttons(relm.clone());

        let mut layout_names = Vec::new();
        for (layout_name, _) in keyboard.views.keys() {
            layout_names.push(layout_name);
        }
        #[cfg(feature = "suggestions")]
        let preferences_button = make_pref_button(relm.clone(), layout_names);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.set_margin_start(0);
        hbox.set_margin_end(0);

        #[cfg(feature = "suggestions")]
        for suggestion_button in suggestion_buttons {
            hbox.add(&suggestion_button);
        }
        #[cfg(feature = "suggestions")]
        hbox.add(&preferences_button);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
        vbox.add(&overlay);

        let window = Window::new(WindowType::Toplevel);
        window.set_property_default_height(720);
        let relm_clone = relm.clone();
        window.connect_configure_event(move |_, configure_event| {
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
        window.add(&vbox);

        let long_press_gesture = GestureLongPress::new(&drawing_area);
        let drag_gesture = GestureDrag::new(&drawing_area);
        let pan_gesture = GesturePan::new(&hbox, gtk::Orientation::Horizontal);
        //long_press_gesture.group(&drag_gesture); //Is the grouping necessary???

        connect_signals(relm, &long_press_gesture, &drag_gesture, &window, &overlay);
        if crate::submitter::wayland::get_layer_shell().is_some() {
            let window_clone = window.clone();
            wayland::layer_shell::make_overlay_layer(window_clone);
        }

        let stream = relm.stream().clone();
        // Create a channel to be able to send a message from another thread.
        let (channel, sender) = Channel::new(move |msg| {
            // This closure is executed whenever a message is received from the sender.
            // We send a message to the current widget.
            stream.emit(msg);
        });
        window.show_all(); // All widgets are visible
        window.hide(); // Keyboard starts out being invisible and is only shown if requested via DBus or input-method

        // Set visible child MUST be called after show_all. Otherwise it takes no effect!
        let (layout_name, view_name) = keyboard.active_view.clone();
        stack.set_visible_child_name(&GridBuilder::make_grid_name(&layout_name, &view_name));
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
                //preferences_button,
                draw_handler,
                stack,
            },
            _gestures: Gestures {
                _long_press_gesture: long_press_gesture,
                _drag_gesture: drag_gesture,
                _pan_gesture: pan_gesture,
            },
            ui_manager,
            _channel: channel,
        }
    }
}

fn connect_signals(
    relm: &relm::Relm<Win>,
    long_press_gesture: &GestureLongPress,
    drag_gesture: &GestureDrag,
    window: &Window,
    _overlay: &Overlay,
) {
    relm::connect!(
        drag_gesture,
        connect_drag_begin(_, x, y),
        &relm,
        Msg::Input((x, y), InputType::Press)
    );

    relm::connect!(
        long_press_gesture,
        connect_pressed(_, x, y), // Long press detected
        relm,
        Msg::Input((x, y), InputType::LongPress)
    );

    relm::connect!(
        drag_gesture,
        connect_drag_update(drag, x, y),
        &relm,
        Msg::Input(
            (
                drag.get_start_point().unwrap().0 + x,
                drag.get_start_point().unwrap().1 + y
            ),
            InputType::Move(Instant::now())
        )
    );

    relm::connect!(
        drag_gesture,
        connect_drag_end(drag, x, y),
        &relm,
        Msg::Input(
            (
                drag.get_start_point().unwrap_or((-0.5, -0.5)).0 + x, // Hack to avoid crashing when long press on button opens popup. Apparently then there is no starting point
                drag.get_start_point().unwrap_or((-0.5, -0.5)).1 + y,
            ),
            InputType::Release
        )
    );

    // Connect the signal `delete_event` to send the `Quit` message.
    relm::connect!(
        relm,
        window,
        connect_delete_event(_, _),
        return (Some(Msg::Quit), Inhibit(false))
    );
    #[cfg(feature = "gesture")]
    relm::connect!(
        relm,
        overlay,
        connect_draw(_, _),
        return (Some(Msg::UpdateDrawBuffer), gtk::Inhibit(false))
    );
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    match provider.load_from_path(directories::CSS_DIRECTORY) {
        Ok(_) => {
            // We give the CssProvided to the default screen so the CSS rules we added
            // can be applied to our window.
            gtk::StyleContext::add_provider_for_screen(
                &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        Err(_) => {
            eprintln! {"No CSS file to customize the keyboard could be loaded. The file might be missing or broken. Using default CSS"}
        }
    }
}

#[cfg(feature = "suggestions")]
fn make_pref_button(relm: relm::Relm<super::Win>, layout_names: Vec<&String>) -> Button {
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
fn make_suggestion_buttons(relm: relm::Relm<super::Win>) -> Vec<Button> {
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
            relm_clone.stream().emit(Msg::Submit(Submission::Text(
                button.get_label().unwrap().to_string(),
            )))
        };

        new_suggestion_button.connect_clicked(suggestion_closure);

        buttons.push(new_suggestion_button);
    }

    buttons
}
