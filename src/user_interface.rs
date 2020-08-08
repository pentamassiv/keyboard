use crate::config::directories;
use crate::config::ui_defaults;
use crate::spacial_model;
use gtk::*;
use gtk::{GestureExt, OverlayExt};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
struct Dot {
    x: f64,
    y: f64,
    time: Instant,
}

struct Input {
    is_long_press: bool,
    path: Vec<Dot>,
}

pub struct Keyboard {
    layout_name: String,
    view_name: String,
    spacial_model: spacial_model::SpacialModel,
}
impl Keyboard {
    fn get_layout_view_name(&self) -> String {
        Self::make_layout_view_name(&self.layout_name, &self.view_name)
    }
    pub fn make_layout_view_name(layout_name: &str, view_name: &str) -> String {
        let mut layout_view_name = String::from(layout_name);
        layout_view_name.push_str("_"); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }
}

pub struct Model {
    keyboard: Keyboard,
    input: Input,
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    Press(f64, f64, Instant),
    LongPress(f64, f64, Instant),
    Swipe(f64, f64, Instant),
    Release(f64, f64, Instant),
    EnterInput(String, bool),
    UpdateDrawBuffer,
    Quit,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
struct Gestures {
    _long_press_gesture: GestureLongPress,
    _drag_gesture: GestureDrag,
}

struct Widgets {
    window: Window,
    label: gtk::Label,
    draw_handler: relm::DrawHandler<DrawingArea>,
    stack: gtk::Stack,
    layout_views: HashMap<String, Grid>,
}

//The gestures are never read but they can't be freed otherwise the gesture detection does not work
pub struct Win {
    relm: relm::Relm<Win>,
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
    fn model(_: &relm::Relm<Self>, _: Self::ModelParam) -> Model {
        Model {
            input: Input {
                is_long_press: false,
                path: Vec::new(),
            },
            keyboard: Keyboard {
                layout_name: "".to_string(),
                view_name: "".to_string(),
                spacial_model: spacial_model::SpacialModel::new(),
            },
        }
    }

    fn subscriptions(&mut self, relm: &relm::Relm<Self>) {
        relm::interval(relm.stream(), 1000, || Msg::UpdateDrawBuffer);
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Press(x, y, time) => {
                self.model.input.path = Vec::new();
                self.model.input.path.push(Dot { x, y, time });
                println!("Press");
            }
            Msg::LongPress(x, y, _) => {
                self.model.input.is_long_press = true;
                println!("LongPress: x: {}, y: {}", x, y);
            }
            Msg::Swipe(x, y, time) => {
                self.model.input.path.push(Dot { x, y, time });
                println!("Drag: x: {}, y: {}, time: {:?}", x, y, time);
            }
            Msg::Release(x, y, time) => {
                let (x_rel, y_rel) = self.get_rel_coordinates(x, y);
                let layout_name = &self.model.keyboard.layout_name;
                let view_name = &self.model.keyboard.view_name;
                let button_to_activate = self.model.keyboard.spacial_model.get_closest_button(
                    layout_name,
                    view_name,
                    x_rel,
                    y_rel,
                );
                if let Some(button_to_activate) = button_to_activate {
                    self.relm.stream().emit(Msg::EnterInput(
                        button_to_activate.get_label().unwrap().to_string(),
                        false,
                    ));
                    button_to_activate.activate();
                }
                println!("Release: x: {}, y: {}, time: {:?}", x, y, time);
                self.model.input.is_long_press = false;
                self.model.input.path = Vec::new();
            }
            Msg::EnterInput(button_label, end_with_space) => {
                println!("Input: {}", button_label);
                self.type_input(&button_label, end_with_space);
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
        //Might have to be called after the show_all() method
        load_css();
        // GTK+ widgets are used normally within a `Widget`.

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        let layouts = crate::layout::LayoutParser::get_layouts();
        let mut layout_views = HashMap::new();
        for (layout_name, layout) in layouts {
            let view_grids = layout.build_button_grids_and_spacial_model(&relm);
            for (view_name, (view_grid, spacial_model_view)) in view_grids {
                let layout_view_name = Keyboard::make_layout_view_name(&layout_name, &view_name);
                stack.add_named(&view_grid, &layout_view_name);
                layout_views.insert(layout_view_name, view_grid);
                model.keyboard.spacial_model.add_spacial_model(
                    &layout_name,
                    &view_name,
                    spacial_model_view,
                )
            }
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

        let suggestion_button_center = gtk::Button::new();
        suggestion_button_center.set_label("sug_c");
        suggestion_button_center.set_hexpand(true);

        let suggestion_button_right = gtk::Button::new();
        suggestion_button_right.set_label("sug_r");
        suggestion_button_right.set_hexpand(true);

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_margin_start(0);
        hbox.set_margin_end(0);
        hbox.add(&suggestion_button_left);
        hbox.add(&suggestion_button_center);
        hbox.add(&suggestion_button_right);

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
        long_press_gesture.group(&drag_gesture); //Is the grouping necessary???

        connect_signals(
            relm,
            &long_press_gesture,
            &drag_gesture,
            &window,
            &overlay,
            &suggestion_button_left,
            &suggestion_button_center,
            &suggestion_button_right,
        );

        window.show_all();

        // Set visible child MUST be called after show_all. Otherwise it takes no effect!
        stack.set_visible_child_name("us_base");
        model.keyboard.view_name = String::from("base");
        model.keyboard.layout_name = String::from("us");
        Win {
            relm: relm.clone(),
            model,
            widgets: Widgets {
                window,
                label,
                draw_handler,
                stack,
                layout_views,
            },
            _gestures: Gestures {
                _long_press_gesture: long_press_gesture,
                _drag_gesture: drag_gesture,
            },
        }
    }
}

impl Win {
    fn type_input(&self, input: &str, end_with_space: bool) {
        let mut label_text = String::from(self.widgets.label.get_text());
        label_text.push_str(&input);
        if end_with_space {
            label_text.push_str(" ");
        }
        self.widgets.label.set_text(&label_text);
    }

    fn get_rel_coordinates(&self, x: f64, y: f64) -> (i32, i32) {
        let allocation = self.widgets.stack.get_allocation();
        let (width, height) = (allocation.width, allocation.height);
        let x_rel = (crate::layout::RESOLUTIONX as f64 * (x / width as f64)) as i32;
        let y_rel = (crate::layout::RESOLUTIONY as f64 * (y / height as f64)) as i32;
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
        if !self.model.input.is_long_press {
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

#[allow(clippy::too_many_arguments)]
fn connect_signals(
    relm: &relm::Relm<Win>,
    long_press_gesture: &GestureLongPress,
    drag_gesture: &GestureDrag,
    window: &Window,
    overlay: &Overlay,
    suggestion_button_left: &Button,
    suggestion_button_center: &Button,
    suggestion_button_right: &Button,
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
            drag.get_start_point().unwrap().0 + x,
            drag.get_start_point().unwrap().1 + y,
            Instant::now()
        )
    );

    relm::connect!(
        relm,
        suggestion_button_left,
        connect_button_press_event(clicked_button, _),
        return (
            Some(Msg::EnterInput(
                clicked_button.get_label().unwrap().to_string(),
                true
            )),
            gtk::Inhibit(false)
        )
    );
    relm::connect!(
        relm,
        suggestion_button_center,
        connect_button_press_event(clicked_button, _),
        return (
            Some(Msg::EnterInput(
                clicked_button.get_label().unwrap().to_string(),
                true
            )),
            gtk::Inhibit(false)
        )
    );
    relm::connect!(
        relm,
        suggestion_button_right,
        connect_button_press_event(clicked_button, _),
        return (
            Some(Msg::EnterInput(
                clicked_button.get_label().unwrap().to_string(),
                true
            )),
            gtk::Inhibit(false)
        )
    );

    relm::connect!(
        relm,
        overlay,
        connect_draw(_, _),
        return (Some(Msg::UpdateDrawBuffer), gtk::Inhibit(false))
    );
    
    // Connect the signal `delete_event` to send the `Quit` message.
    relm::connect!(
        relm,
        window,
        connect_delete_event(_, _),
        return (Some(Msg::Quit), Inhibit(false))
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
