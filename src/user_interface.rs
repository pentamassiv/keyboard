use crate::config::directories;
use crate::config::ui_defaults;
use gdk::EventMask;
use gtk::prelude::WidgetExtManual;
use gtk::*;

#[derive(Clone)]
struct Dot {
    position: (f64, f64),
    time: u32,
}

impl Dot {
    fn new(position: (f64, f64), time: u32) -> Self {
        Dot { position, time }
    }
}

pub struct Model {
    draw_handler: relm::DrawHandler<DrawingArea>,
    dots: Vec<Dot>,
    is_pressed: bool,
    layouts: std::collections::HashMap<String, super::layout::Layout>,
}

impl Model {
    fn erase_path(&mut self) {
        let context = self.draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
    }

    fn draw_path(&mut self) {
        let context = self.draw_handler.get_context();
        self.erase_path();
        context.set_operator(cairo::Operator::Over);
        context.set_source_rgba(
            ui_defaults::PATHCOLOR.0,
            ui_defaults::PATHCOLOR.1,
            ui_defaults::PATHCOLOR.2,
            ui_defaults::PATHCOLOR.3,
        );
        //let mut time_now = 0;
        for dot in self.dots.iter().rev().take(ui_defaults::PATHLENGTH) {
            // Only draw the last dots within a certain time period. Works but there would have to be a draw signal in a regular interval to make it look good
            //if dot.time > time_now {
            //    time_now = dot.time
            //}
            //if time_now - dot.time < PATHFADINGTIME {
            context.line_to(dot.position.0, dot.position.1);
            //} else {
            //    break;
            //}
        }
        context.set_line_width(ui_defaults::PATHWIDTH);
        context.stroke();
    }
}

#[derive(relm_derive::Msg)]
pub enum Msg {
    Press,
    Release,
    MovePointer((f64, f64), u32),
    Quit,
    UpdateDrawBuffer,
    SuggestionPress(String),
}

pub struct Win {
    drawing_area: gtk::DrawingArea,
    layout_stack: gtk::Stack,
    label: gtk::Label,
    model: Model,
    window: Window,
}

impl relm::Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = std::collections::HashMap<String, super::layout::Layout>;
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    // Return the initial model.
    fn model(
        _: &relm::Relm<Self>,
        layouts: std::collections::HashMap<String, super::layout::Layout>,
    ) -> Model {
        Model {
            draw_handler: relm::DrawHandler::new().expect("draw handler"),
            dots: Vec::new(),
            is_pressed: false,
            layouts,
        }
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Press => {
                self.model.is_pressed = true;
            }
            Msg::SuggestionPress(button_label) => {
                let mut label_text = String::from(self.label.get_text());
                label_text.push_str(&button_label);
                label_text.push_str(" ");
                self.label.set_text(&label_text);
                // Delete the following, its just for testing
                if &button_label == "sug_but_r" {
                    self.layout_stack.set_visible_child_name("us");
                } else {
                    self.layout_stack.set_visible_child_name("de");
                }
            }
            Msg::Release => {
                self.model.is_pressed = false;
                let mut label_text = String::from(self.label.get_text());
                label_text.push_str(&self.model.dots.len().to_string());
                label_text.push_str(" ");
                self.label.set_text(&label_text);
                self.model.erase_path();
                self.model.dots = Vec::new();
            }
            Msg::MovePointer(pos, time) => {
                if self.model.is_pressed {
                    self.model.dots.push(Dot::new(pos, time));
                }
            }
            Msg::Quit => gtk::main_quit(),
            Msg::UpdateDrawBuffer => {
                self.model.draw_path();
            }
        }
    }
}

impl relm::Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    // Create the widgets.
    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        //Might have to be called after the show_all() method
        load_css();
        // GTK+ widgets are used normally within a `Widget`.

        let layout_stack = gtk::Stack::new();
        layout_stack.set_transition_type(gtk::StackTransitionType::None);

        for (layout_name, layout) in &model.layouts {
            let view_stack = gtk::Stack::new();
            let view_grids = layout.build_button_grid();
            for (view_name, view_grid) in view_grids {
                view_stack.add_named(&view_grid, &view_name);
            }
            view_stack.set_transition_type(gtk::StackTransitionType::None);
            layout_stack.add_named(&view_stack, &layout_name);
        }

        let drawing_area = gtk::DrawingArea::new();
        let overlay = gtk::Overlay::new();
        overlay.add(&layout_stack);
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

        // Connect the signal `delete_event` to send the `Quit` message.
        relm::connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        drawing_area.add_events(EventMask::POINTER_MOTION_MASK);
        drawing_area.add_events(EventMask::BUTTON_PRESS_MASK);
        drawing_area.add_events(EventMask::BUTTON_RELEASE_MASK);

        suggestion_button_left.add_events(EventMask::BUTTON_PRESS_MASK);
        suggestion_button_center.add_events(EventMask::BUTTON_PRESS_MASK);
        suggestion_button_right.add_events(EventMask::BUTTON_PRESS_MASK);

        relm::connect!(
            relm,
            suggestion_button_left,
            connect_button_press_event(clicked_button, _),
            return (
                Some(Msg::SuggestionPress(
                    clicked_button.get_label().unwrap().to_string()
                )),
                gtk::Inhibit(false)
            )
        );
        relm::connect!(
            relm,
            suggestion_button_center,
            connect_button_press_event(clicked_button, _),
            return (
                Some(Msg::SuggestionPress(
                    clicked_button.get_label().unwrap().to_string()
                )),
                gtk::Inhibit(false)
            )
        );
        relm::connect!(
            relm,
            suggestion_button_right,
            connect_button_press_event(clicked_button, _),
            return (
                Some(Msg::SuggestionPress(
                    clicked_button.get_label().unwrap().to_string()
                )),
                gtk::Inhibit(false)
            )
        );

        relm::connect!(
            relm,
            overlay,
            connect_motion_notify_event(_, event),
            return (
                Some(Msg::MovePointer(event.get_position(), event.get_time())),
                gtk::Inhibit(false)
            )
        );

        relm::connect!(
            relm,
            overlay,
            connect_button_press_event(_, _),
            return (Some(Msg::Press), gtk::Inhibit(false))
        );

        relm::connect!(
            relm,
            overlay,
            connect_button_release_event(_, _),
            return (Some(Msg::Release), gtk::Inhibit(false))
        );

        relm::connect!(
            relm,
            overlay,
            connect_draw(_, _),
            return (Some(Msg::UpdateDrawBuffer), gtk::Inhibit(false))
        );

        window.show_all();

        Win {
            drawing_area,
            layout_stack,
            label,
            model,
            window,
        }
    }
    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);
    }
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
