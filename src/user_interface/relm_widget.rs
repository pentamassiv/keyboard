use super::*;

impl relm::Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    // Create the widgets.
    fn view(relm: &relm::Relm<Self>, mut model: Self::Model) -> Self {
        load_css();

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        let layout_meta = keyboard::parser::LayoutYamlParser::get_layouts();
        let grids = model.keyboard.init(relm, layout_meta);
        for (grid_name, grid) in grids {
            stack.add_named(&grid, &grid_name);
        }

        let drawing_area = gtk::DrawingArea::new();
        let mut draw_handler = relm::DrawHandler::new().expect("draw handler");
        draw_handler.init(&drawing_area);
        let overlay = gtk::Overlay::new();
        overlay.add(&stack);
        overlay.add_overlay(&drawing_area);

        let suggestion_button_left = gtk::Button::new();
        suggestion_button_left.set_label("sug_l");
        suggestion_button_left.set_hexpand(true);
        suggestion_button_left.set_focus_on_click(false);

        let suggestion_button_center = gtk::Button::new();
        suggestion_button_center.set_label("sug_c");
        suggestion_button_center.set_hexpand(true);
        suggestion_button_center.set_focus_on_click(false);

        let suggestion_button_right = gtk::Button::new();
        suggestion_button_right.set_label("sug_r");
        suggestion_button_right.set_hexpand(true);
        suggestion_button_right.set_focus_on_click(false);

        let relm_copy_left = relm.clone();
        let suggestion_closure_left = move |button: &gtk::Button| {
            relm_copy_left.stream().emit(Msg::Submit(Submission::Text(
                button.get_label().unwrap().to_string(),
            )))
        };
        let relm_copy_center = relm.clone();
        let suggestion_closure_center = move |button: &gtk::Button| {
            relm_copy_center.stream().emit(Msg::Submit(Submission::Text(
                button.get_label().unwrap().to_string(),
            )))
        };
        let relm_copy_right = relm.clone();
        let suggestion_closure_right = move |button: &gtk::Button| {
            relm_copy_right.stream().emit(Msg::Submit(Submission::Text(
                button.get_label().unwrap().to_string(),
            )))
        };

        suggestion_button_left.connect_clicked(suggestion_closure_left);
        suggestion_button_center.connect_clicked(suggestion_closure_center);
        suggestion_button_right.connect_clicked(suggestion_closure_right);

        let preferences_button = gtk::Button::new();
        preferences_button.set_label("pref");
        preferences_button.set_hexpand(true);
        preferences_button.set_focus_on_click(false);

        let pref_popover = gtk::Popover::new(Some(&preferences_button));
        let pref_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        pref_popover.add(&pref_vbox);
        let mut tmp_layouts = HashMap::new();
        for (layout_name, _) in model.keyboard.views.keys() {
            tmp_layouts.insert(layout_name, ());
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

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_margin_start(0);
        hbox.set_margin_end(0);
        hbox.add(&suggestion_button_left);
        hbox.add(&suggestion_button_center);
        hbox.add(&suggestion_button_right);
        hbox.add(&preferences_button);

        let frame = gtk::Frame::new(None);
        frame.add(&hbox);

        let vbox = gtk::Box::new(Orientation::Vertical, 2);
        vbox.add(&frame);
        vbox.add(&overlay);

        let window = Window::new(WindowType::Toplevel);
        window.set_property_default_height(720);
        let relm_clone = relm.clone();
        window.connect_configure_event(move |_, configure_event| {
            let (width, _) = configure_event.get_size();
            let mode = if width == 720 {
                Mode::Landscape
            } else {
                Mode::Portrait
            };
            relm_clone.stream().emit(Msg::ChangeUIMode(mode));
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
        let (layout_name, view_name) = model.keyboard.get_view_name();
        stack.set_visible_child_name(&keyboard::Keyboard::make_view_name(
            &layout_name,
            &view_name,
        ));
        let ui_manager = UIManager::new(
            sender,
            window.clone(),
            stack.clone(),
            (layout_name, view_name),
        );
        Win {
            relm: relm.clone(),
            model,
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
    overlay: &Overlay,
) {
    relm::connect!(
        drag_gesture,
        connect_drag_begin(_, x, y),
        &relm,
        Msg::Press(x, y, Instant::now())
    );

    relm::connect!(
        long_press_gesture,
        connect_pressed(_, x, y), // Long press detected
        relm,
        Msg::LongPress(x, y, Instant::now())
    );

    relm::connect!(
        drag_gesture,
        connect_drag_update(drag, x, y),
        &relm,
        Msg::Swipe(
            drag.get_start_point().unwrap().0 + x,
            drag.get_start_point().unwrap().1 + y,
            Instant::now()
        )
    );

    relm::connect!(
        drag_gesture,
        connect_drag_end(drag, x, y),
        &relm,
        Msg::Release(
            drag.get_start_point().unwrap_or((-0.5, -0.5)).0 + x, // Hack to avoid crashing when long press on button opens popup. Apparently then there is no starting point
            drag.get_start_point().unwrap_or((-0.5, -0.5)).1 + y,
            Instant::now(),
        )
    );

    // Connect the signal `delete_event` to send the `Quit` message.
    relm::connect!(
        relm,
        window,
        connect_delete_event(_, _),
        return (Some(Msg::Quit), Inhibit(false))
    );

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
