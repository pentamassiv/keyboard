use gdk::EventMask;
use gtk::Align::*;
use gtk::Orientation::*;

use gtk::*;
use gtk::{
    prelude::WidgetExtManual, BoxExt, ButtonExt, CssProviderExt, DrawingArea, GtkWindowExt,
    Inhibit, LabelExt, OrientableExt, WidgetExt,
};

//use cairo::{Antialias, Context, LineCap};
use relm::{DrawHandler, Relm, Widget};
use relm_derive::widget;
use relm_derive::Msg;

use self::Msg::*;

// Defines const for drawn path
const PATHCOLOR: (f64, f64, f64, f64) = (0.105, 0.117, 0.746, 0.9);
const PATHLENGTH: usize = 100;
const PATHWIDTH: f64 = 4.5;
const PATHFADINGTIME: u32 = 400;

const CSS_DIRECTORY: &str = "./theming/style.css";

#[derive(Clone)]
struct Dot {
    position: (f64, f64),
    time: u32,
}

impl Dot {
    fn generate(position: (f64, f64), time: u32) -> Self {
        Dot { position, time }
    }
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    dots: Vec<Dot>,
    is_pressed: bool,
    layouts: std::collections::HashMap<String, super::layout::Layout>,
}

#[derive(Msg)]
pub enum Msg {
    Press,
    Release,
    MovePointer((f64, f64), u32),
    Quit,
    UpdateDrawBuffer,
    SuggestionPress(String),
}

#[allow(clippy::redundant_field_names)]
#[widget]
impl Widget for Win {
    fn model(
        _: &Relm<Self>,
        layouts: std::collections::HashMap<String, super::layout::Layout>,
    ) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            dots: Vec::new(),
            is_pressed: false,
            layouts,
        }
    }

    view! {
        gtk::Window {
            property_default_height: 720,
            property_default_width: 360,
            #[name="vbox"]
            gtk::Box {
                orientation: Vertical,
                spacing: 2,
                #[name="label"]
                gtk::Label {
                    margin_start: 5,
                    margin_end: 5,
                    text: "",
                    line_wrap: true,
                    child: {
                        expand: true,
                    },
                },
                gtk::Frame{
                    gtk::Box {
                        orientation: Horizontal,
                        halign: Fill,
                        margin_start: 0,
                        margin_end: 0,
                        spacing: 0,
                        #[name="suggestion_button_left"]
                        gtk::Button {
                            label: "sug_but_l",
                            button_press_event(clicked_button, event) => (SuggestionPress(clicked_button.get_label().unwrap().to_string()), Inhibit(false)),
                            child: {
                                expand: true,
                            },
                        },
                        #[name="suggestion_button_center"]
                        gtk::Button {
                            label: "sug_but_c",
                            button_press_event(clicked_button, event) => (SuggestionPress(clicked_button.get_label().unwrap().to_string()), Inhibit(false)),
                            child: {
                                expand: true,
                            },
                        },
                        #[name="suggestion_button_right"]
                        gtk::Button {
                            label: "sug_but_r",
                            button_press_event(clicked_button, event) => (SuggestionPress(clicked_button.get_label().unwrap().to_string()), Inhibit(false)),
                            child: {
                                expand: true,
                            },
                        },
                    },
                },
                #[name="overlay"]
                gtk::Overlay {
                    hexpand:true,
                    motion_notify_event(_, event) => (MovePointer(event.get_position(), event.get_time()), Inhibit(false)),
                    button_press_event(_, event) => (Press, Inhibit(false)),
                    button_release_event(_, event) => (Release, Inhibit(false)),
                    draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
                    #[name="layout_stack"]
                    gtk::Stack {
                        transition_type: gtk::StackTransitionType::None,
                        valign: Fill,
                        hexpand:true,
                    },
                }
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        self.label.set_size_request(360, 200);
        let drawing_area = gtk::DrawingArea::new();
        self.model.draw_handler.init(&drawing_area);
        drawing_area.add_events(EventMask::POINTER_MOTION_MASK);
        drawing_area.add_events(EventMask::BUTTON_PRESS_MASK);
        drawing_area.add_events(EventMask::BUTTON_RELEASE_MASK);
        self.suggestion_button_left
            .add_events(EventMask::BUTTON_PRESS_MASK);
        self.suggestion_button_center
            .add_events(EventMask::BUTTON_PRESS_MASK);
        self.suggestion_button_right
            .add_events(EventMask::BUTTON_PRESS_MASK);
        self.overlay.add_overlay(&drawing_area);
        self.load_keys_from_all_layouts();
        self.overlay.show_all();
        load_css();
    }

    fn load_keys_from_all_layouts(&self) {
        for (layout_name, layout) in &self.model.layouts {
            let view_stack = gtk::Stack::new();
            view_stack.set_transition_type(gtk::StackTransitionType::None);
            for (view_name, view) in layout.get_buttons() {
                let button_vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
                button_vbox.set_halign(Fill);
                for row in view {
                    let button_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                    button_hbox.set_halign(Fill);
                    for button in row {
                        let insert_button = button;
                        insert_button.set_halign(Fill);
                        insert_button.set_hexpand(true);
                        button_hbox.add(&insert_button);
                    }
                    button_vbox.add(&button_hbox);
                }
                view_stack.add_named(&button_vbox, &view_name);
            }
            self.layout_stack.add_named(&view_stack, &layout_name);
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Press => {
                self.model.is_pressed = true;
            }
            SuggestionPress(button_label) => {
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
            Release => {
                self.model.is_pressed = false;
                let mut label_text = String::from(self.label.get_text());
                //label_text.push_str(
                //    &self
                //        .suggestion_button_right
                //        .get_allocated_size()
                //        .to_string(),
                //);
                label_text.push_str(&self.model.dots.len().to_string());
                label_text.push_str(" ");
                self.label.set_text(&label_text);
                self.erase_path();
                self.model.dots = Vec::new();
                //self.model.draw_handler.get_context().show_page();
            }
            MovePointer(pos, time) => {
                if self.model.is_pressed {
                    self.model.dots.push(Dot::generate(pos, time));
                }
            }
            Quit => gtk::main_quit(),
            UpdateDrawBuffer => {
                self.draw_path();
            }
        }
    }

    fn erase_path(&mut self) {
        let context = self.model.draw_handler.get_context();
        context.set_operator(cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint();
    }

    fn draw_path(&mut self) {
        let context = self.model.draw_handler.get_context();
        self.erase_path();
        context.set_operator(cairo::Operator::Over);
        context.set_source_rgba(PATHCOLOR.0, PATHCOLOR.1, PATHCOLOR.2, PATHCOLOR.3);
        let mut time_now = 0;
        for dot in self.model.dots.iter().rev().take(PATHLENGTH) {
            if dot.time > time_now {
                time_now = dot.time
            }
            if time_now - dot.time < PATHFADINGTIME {
                context.line_to(dot.position.0, dot.position.1);
            } else {
                break;
            }
        }
        context.set_line_width(PATHWIDTH);
        context.stroke();
    }
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    match provider.load_from_path(CSS_DIRECTORY) {
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
