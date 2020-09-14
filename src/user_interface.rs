use super::submitter::*;
use crate::config::directories;
use crate::config::ui_defaults;
use crate::keyboard;
use crate::keyboard::{EmitUIMsg, KeyAction, KeyEvent, UIMsg};
use gtk::OverlayExt;
use gtk::*;
use std::collections::HashMap;
use std::time::Instant;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

#[derive(Clone)]
struct Dot {
    x: f64,
    y: f64,
    time: Instant,
}

struct Input {
    input_type: KeyEvent,
    path: Vec<Dot>,
}

pub struct Model {
    keyboard: crate::keyboard::Keyboard,
    input: Input,
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    Press(f64, f64, Instant),
    LongPress(f64, f64, Instant),
    Swipe(f64, f64, Instant),
    Release(f64, f64, Instant),
    Submit(Submission),
    Visable(bool),
    HintPurpose(ContentHint, ContentPurpose),
    SwitchView(String),
    SwitchLayout(String),
    PollEvents,
    UpdateDrawBuffer,
    Quit,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
struct Gestures {
    _long_press_gesture: GestureLongPress,
    _drag_gesture: GestureDrag,
    _pan_gesture: GesturePan,
}

struct Widgets {
    window: Window,
    label: gtk::Label,
    draw_handler: relm::DrawHandler<DrawingArea>,
    stack: gtk::Stack,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
pub struct Win {
    pub relm: relm::Relm<Win>,
    model: Model,
    widgets: Widgets,
    _gestures: Gestures,
}

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
        relm::interval(relm.stream(), 1000, || Msg::PollEvents);
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Press(_, _, _) => {
                self.model.input.input_type = KeyEvent::ShortPress;
                //self.model.input.path = Vec::new();
                //self.model.input.path.push(Dot { x, y, time });
                //println!("Press");
            }
            Msg::LongPress(x, y, _) => {
                self.model.input.input_type = KeyEvent::LongPress;
                self.activate_button(x, y);
                //println!("LongPress: x: {}, y: {}", x, y);
            }
            Msg::Swipe(x, y, time) => {
                if !(self.model.input.input_type == KeyEvent::LongPress) {
                    self.model.input.input_type = KeyEvent::Swipe;
                    self.model.input.path.push(Dot { x, y, time });
                }
                //println!("Drag: x: {}, y: {}, time: {:?}", x, y, time);
            }
            Msg::Release(x, y, time) => {
                match self.model.input.input_type {
                    KeyEvent::ShortPress => {
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
            Msg::Visable(visable) => println!("Visable: {}", visable),
            Msg::HintPurpose(content_hint, content_purpose) => println!(
                "ContentHint: {:?}, ContentPurpose: {:?}",
                content_hint, content_purpose
            ),
            Msg::SwitchLayout(new_layout) => {
                self.widgets.stack.set_visible_child_name(
                    &crate::keyboard::Keyboard::make_view_name(&new_layout, "base"),
                );
                self.model.keyboard.active_view = (new_layout, "base".to_string());
            }
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
                    Some(crate::user_interface::Msg::SwitchLayout(
                        clicked_button.get_label().unwrap().to_string()
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

        let label = gtk::Label::new(None);
        label.set_margin_start(5);
        label.set_margin_end(5);
        label.set_line_wrap(true);
        label.set_vexpand(true);
        let vbox = gtk::Box::new(Orientation::Vertical, 2);
        vbox.add(&label);
        vbox.add(&frame);
        vbox.add(&overlay);

        let window = Window::new(WindowType::Toplevel);
        window.set_property_default_height(720);
        window.add(&vbox);

        let long_press_gesture = GestureLongPress::new(&drawing_area);
        let drag_gesture = GestureDrag::new(&drawing_area);
        let pan_gesture = GesturePan::new(&hbox, gtk::Orientation::Horizontal);
        //long_press_gesture.group(&drag_gesture); //Is the grouping necessary???

        connect_signals(relm, &long_press_gesture, &drag_gesture, &window, &overlay);
        if super::submitter::wayland::get_layer_shell().is_some() {
            let window_clone = window.clone();
            wayland::layer_shell::make_overlay_layer(window_clone);
        }
        window.show_all();

        // Set visible child MUST be called after show_all. Otherwise it takes no effect!
        let (layout_name, view_name) = model.keyboard.get_view_name();
        stack.set_visible_child_name(&keyboard::Keyboard::make_view_name(
            &layout_name,
            &view_name,
        ));
        Win {
            relm: relm.clone(),
            model,
            widgets: Widgets {
                window,
                label,
                //preferences_button,
                draw_handler,
                stack,
            },
            _gestures: Gestures {
                _long_press_gesture: long_press_gesture,
                _drag_gesture: drag_gesture,
                _pan_gesture: pan_gesture,
            },
        }
    }
}

impl Win {
    fn activate_button(&self, x: f64, y: f64) {
        let (x_rel, y_rel) = self.get_rel_coordinates(x, y);
        let (layout_name, view_name) = &self.model.keyboard.get_view_name();
        if let Some(key_to_activate) =
            self.model
                .keyboard
                .get_closest_key(layout_name, view_name, x_rel, y_rel)
        {
            key_to_activate.activate(self, &self.model.input.input_type);
        }
    }

    fn get_rel_coordinates(&self, x: f64, y: f64) -> (i32, i32) {
        let allocation = self.widgets.stack.get_allocation();
        let (width, height) = (allocation.width, allocation.height);
        let x_rel = (crate::keyboard::RESOLUTIONX as f64 * (x / width as f64)) as i32;
        let y_rel = (crate::keyboard::RESOLUTIONY as f64 * (y / height as f64)) as i32;
        (x_rel, y_rel)
    }

    fn erase_path(&mut self) {
        let context = self.widgets.draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
    }

    fn draw_path(&mut self) {
        self.erase_path();
        if self.model.input.input_type == KeyEvent::Swipe {
            let context = self.widgets.draw_handler.get_context();
            context.set_operator(cairo::Operator::Over);
            context.set_source_rgba(
                ui_defaults::PATHCOLOR.0,
                ui_defaults::PATHCOLOR.1,
                ui_defaults::PATHCOLOR.2,
                ui_defaults::PATHCOLOR.3,
            );
            let max_duration = std::time::Duration::from_millis(ui_defaults::PATHFADINGDURATION);
            for dot in self
                .model
                .input
                .path
                .iter()
                .rev()
                .take(ui_defaults::PATHLENGTH)
            {
                // Only draw the last dots within a certain time period. Works but there would have to be a draw signal in a regular interval to make it look good
                if dot.time.elapsed() < max_duration {
                    context.line_to(dot.x, dot.y);
                } else {
                    break;
                }
            }
            context.set_line_width(ui_defaults::PATHWIDTH);
            context.stroke();
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

// Needed because Rust does not allow implementing a trait for a struct if neighter of them is defined in the scope
// Relm is from the relm crate and EmitUIMsg is from another module
pub struct MessagePipe {
    relm: relm::Relm<crate::user_interface::Win>,
}

impl MessagePipe {
    fn new(relm: relm::Relm<Win>) -> MessagePipe {
        MessagePipe { relm }
    }
}

impl EmitUIMsg for MessagePipe {
    fn emit(&self, message: UIMsg) {
        match message {
            UIMsg::SwitchView(view) => {
                self.relm.stream().emit(Msg::SwitchView(view));
            }
            UIMsg::Visable(visable) => {
                println!("Relm: visability: {}", visable);
                self.relm.stream().emit(Msg::Visable(visable));
            }
            UIMsg::HintPurpose(content_hint, content_purpose) => println!("Relm: contentpurpose"),
            UIMsg::SwitchLayout(layout) => println!("Relm: switch layout"),
            _ => {}
        }
    }
}
