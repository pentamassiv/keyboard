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
            gesture: GestureModel::new(relm.clone()),
        }
    }

    fn subscriptions(&mut self, relm: &relm::Relm<Self>) {
        relm::interval(relm.stream(), 100, || Msg::PollEvents);
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::GestureSignal(x, y, gesture_signal) => {
                self.model.gesture.handle(x, y, gesture_signal);
            }
            Msg::Interaction((x, y), interaction) => {
                let (x, y) = self.get_rel_coordinates(x, y);
                self.keyboard.input(x, y, interaction);
            }
            Msg::ButtonInteraction(key_id, tap_motion) => {
                let (layout, view) = self.ui_manager.current_layout_view.clone();
                if let Some((button, _)) = self.key_refs.get(&(layout, view, key_id)) {
                    button.set_active(tap_motion == TapMotion::Press);
                    #[cfg(feature = "haptic-feedback")]
                    self.ui_manager
                        .haptic_feedback(tap_motion == TapMotion::Press);
                }
            }
            Msg::OpenPopup(key_id) => {
                let (layout, view) = self.ui_manager.current_layout_view.clone();
                if let Some((button, popover)) = self.key_refs.get(&(layout, view, key_id)) {
                    button.set_active(false);
                    if let Some(popover) = popover {
                        popover.show_all();
                    } else {
                        error!("The button does not have a popup to open");
                    }
                } else {
                    error!("UI does not know the key id");
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
