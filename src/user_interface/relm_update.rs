use super::*;

impl relm::Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    // Return the initial model.
    fn model(relm: &relm::Relm<Self>, _: Self::ModelParam) -> Model {
        Model {
            input: Input {
                input_type: KeyEvent::ShortPress,
                path: Vec::new(),
            },
            keyboard: keyboard::Keyboard::new(MessagePipe::new(relm.clone())),
        }
    }

    fn subscriptions(&mut self, relm: &relm::Relm<Self>) {
        relm::interval(relm.stream(), 1000, || Msg::UpdateDrawBuffer);
        relm::interval(relm.stream(), 100, || Msg::PollEvents);
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Press(_, _, _) => {
                self.model.input.input_type = KeyEvent::ShortPress;
            }
            Msg::LongPress(x, y, _) => {
                self.model.input.input_type = KeyEvent::LongPress;
                // self.dbus_service.haptic_feedback(); // Not working reliably
                self.activate_button(x, y);
            }
            Msg::Swipe(x, y, time) => {
                if !(self.model.input.input_type == KeyEvent::LongPress) {
                    self.model.input.input_type = KeyEvent::Swipe;
                    self.model.input.path.push(Dot { x, y, time });
                }
            }
            Msg::Release(x, y, time) => {
                match self.model.input.input_type {
                    KeyEvent::ShortPress => {
                        // self.dbus_service.haptic_feedback(); // Not working reliably
                        self.activate_button(x, y);
                    }
                    KeyEvent::LongPress => {
                        //println!("LongPress");
                    }
                    KeyEvent::Swipe => {
                        //println!("Swipe");
                    }
                }
                //println!("Release: x: {}, y: {}, time: {:?}", x, y, time);
                self.model.input.path = Vec::new();
            }
            Msg::Submit(submission) => self.model.keyboard.submit(submission),
            Msg::SwitchView(new_view) => {
                let layout_name = &self.model.keyboard.active_view.0;
                self.widgets.stack.set_visible_child_name(
                    &crate::keyboard::Keyboard::make_view_name(layout_name, &new_view),
                );
                self.model.keyboard.active_view = (layout_name.to_string(), new_view);
            }
            Msg::Visible(new_visibility) => {
                println!("Msg visiblility: {}", new_visibility);
                if new_visibility {
                    self.widgets.window.show();
                } else {
                    self.widgets.window.hide();
                }
                self.dbus_service.change_visibility(new_visibility);
            }
            Msg::HintPurpose(content_hint, content_purpose) => println!(
                "ContentHint: {:?}, ContentPurpose: {:?}",
                content_hint, content_purpose
            ),
            Msg::SwitchLayout(new_layout) => {
                if self.widgets.stack.get_child_by_name(&new_layout).is_some() {
                    self.widgets.stack.set_visible_child_name(
                        &crate::keyboard::Keyboard::make_view_name(&new_layout, "base"),
                    );
                    self.model.keyboard.active_view = (new_layout, "base".to_string());
                } else {
                    println!("The requested layout {} does not exist", new_layout);
                }
            }
            Msg::SwitchMode(mode) => match mode {
                Mode::Landscape => println!("Mode changed to Landscape"),
                Mode::Portrait => println!("Mode changed to Portrait"),
            },
            Msg::PollEvents => {
                self.model.keyboard.fetch_events();
            }
            Msg::UpdateDrawBuffer => {
                self.draw_path();
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}
