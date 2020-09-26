use super::*;

impl relm::Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    // Return the initial model.
    fn model(_: &relm::Relm<Self>, _: Self::ModelParam) -> Model {
        Model { path: Vec::new() }
    }

    fn subscriptions(&mut self, relm: &relm::Relm<Self>) {
        relm::interval(relm.stream(), 100, || Msg::PollEvents);
        #[cfg(feature = "gesture")]
        relm::interval(relm.stream(), 1000, || Msg::UpdateDrawBuffer);
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Input((x, y), input_type) => {
                if let InputType::Move(time) = input_type {
                    self.model.path.push(Point { x, y, time });
                }
                if let InputType::Release = input_type {
                    self.model.path = Vec::new();
                }
                let (x, y) = self.get_rel_coordinates(x, y);
                self.keyboard.input(x, y, input_type);
            }
            Msg::ButtonInteraction(key_id, key_motion) => {
                // Should never be possible to fail
                let (layout, view) = self.ui_manager.current_layout_view.clone();
                if let Some((button, _)) = self.key_refs.get(&(layout, view, key_id)) {
                    // TODO: Check what the ui is supposed to do when a button is activated
                    match key_motion {
                        KeyMotion::Press => {
                            button.set_active(true);
                        }
                        KeyMotion::Release => {
                            button.set_active(false);
                        } //button.deactivate(),
                    }

                    #[cfg(feature = "haptic-feedback")]
                    self.dbus_service.haptic_feedback();
                }
            }
            Msg::OpenPopup(key_id) => {
                let (layout, view) = self.ui_manager.current_layout_view.clone();
                if let Some((button, popover)) = self.key_refs.get(&(layout, view, key_id)) {
                    button.set_active(false);
                    if let Some(popover) = popover {
                        popover.show_all();
                    }
                }
            }
            Msg::SubmitText(text) => self.keyboard.submit_text(text),
            Msg::Visible(new_visibility) => {
                self.ui_manager.change_visibility(new_visibility);
            }
            Msg::HintPurpose(content_hint, content_purpose) => self
                .ui_manager
                .change_hint_purpose(content_hint, content_purpose),
            Msg::ChangeUILayoutView(layout, view) => {
                println!("new_layout: {:?}, new_view: {:?}", layout, view);
                let _ = self.ui_manager.change_layout_view(layout, view); // Result not relevant
            }
            Msg::ChangeKBLayoutView(layout, view) => {
                self.keyboard.active_view = (layout, view);
            }
            Msg::ChangeUIOrientation(mode) => self.ui_manager.change_orientation(mode),
            Msg::PollEvents => {
                self.keyboard.fetch_events();
            }
            #[cfg(feature = "gesture")]
            Msg::UpdateDrawBuffer => {
                self.draw_path();
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}
