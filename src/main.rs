/*
 * Copyright (c) 2020 Grell, Robin <grellr@hochschule-trier.de>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use gdk::EventMask;
use gtk::Align::*;
use gtk::Orientation::*;

use gtk::*;
use gtk::{
    prelude::WidgetExtManual, BoxExt, ButtonExt, ContainerExt, CssProviderExt, DrawingArea,
    GtkWindowExt, Inhibit, LabelExt, NotebookExt, OrientableExt, TextBuffer, TextBufferExt,
    TextTag, TextTagTable, TextTagTableExt, TextViewExt, WidgetExt,
};

//use cairo::{Antialias, Context, LineCap};
use relm::{DrawHandler, Widget};
use relm_derive::widget;
use relm_derive::Msg;

use self::Msg::*;

// Defines color of path
const PATHCOLOR: (f64, f64, f64, f64) = (0.105, 0.117, 0.746, 0.9);
const BACKGROUNDCOLOR: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 0.0); // Background is translucent so the buttons beneight are visible

struct Dot {
    position: (f64, f64),
}

impl Dot {
    fn generate(position: (f64, f64)) -> Self {
        Dot { position }
    }
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    dots: Vec<Dot>,
    is_pressed: bool,
}

#[derive(Msg)]
pub enum Msg {
    Press,
    Release,
    MovePointer((f64, f64)),
    Quit,
    UpdateDrawBuffer,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            dots: Vec::new(),
            is_pressed: false,
        }
    }

    view! {
        gtk::Window {
            property_default_height: 720,
            property_default_width: 360,
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
                            child: {
                                expand: true,
                            },
                        },
                        #[name="suggestion_button_center"]
                        gtk::Button {
                            label: "sug_but_c",
                            child: {
                                expand: true,
                            },
                        },
                        #[name="suggestion_button_right"]
                        gtk::Button {
                            label: "sug_but_r",
                            child: {
                                expand: true,
                            },
                        },
                    },
                },
                #[name="drawing_area"]
                gtk::DrawingArea {
                    child: {
                        expand: true,
                    },
                    draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
                    motion_notify_event(_, event) => (MovePointer(event.get_position()), Inhibit(false)),
                    button_press_event(_, event) => (Press, Inhibit(false)),
                    button_release_event(_, event) => (Release, Inhibit(false))
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);
        self.drawing_area.add_events(EventMask::POINTER_MOTION_MASK);
        self.drawing_area.add_events(EventMask::BUTTON_PRESS_MASK);
        self.drawing_area.add_events(EventMask::BUTTON_RELEASE_MASK);
        self.label.set_size_request(360, 200);
        load_css();
    }

    fn update(&mut self, event: Msg) {
        match event {
            Press => {
                self.model.is_pressed = true;
                //self.model.draw_handler.get_context().save();
            }
            Release => {
                self.model.is_pressed = false;
                self.model.dots = Vec::new();
                //self.model.draw_handler.get_context().restore();
                //self.reset_keyboard();
                let mut label_text = String::from(self.label.get_text());
                label_text.push_str(&"word ");
                self.label.set_text(&label_text);
            }
            MovePointer(pos) => {
                if self.model.is_pressed {
                    self.model.dots.push(Dot::generate(pos));
                }
            }
            Quit => gtk::main_quit(),
            UpdateDrawBuffer => {
                self.draw_path();
            }
        }
    }

    fn reset_keyboard(&mut self) {
        let context = self.model.draw_handler.get_context();
        //Set color of background to white
        context.set_source_rgba(
            BACKGROUNDCOLOR.0,
            BACKGROUNDCOLOR.1,
            BACKGROUNDCOLOR.2,
            BACKGROUNDCOLOR.3,
        );
        context.paint();
    }

    fn draw_path(&mut self) {
        let context = self.model.draw_handler.get_context();
        //context.set_antialias(Antialias::Best);
        //context.set_line_cap(LineCap::Round); //Default is LineCap::Butt
        context.set_source_rgba(PATHCOLOR.0, PATHCOLOR.1, PATHCOLOR.2, PATHCOLOR.3);
        for dot in &self.model.dots {
            context.line_to(dot.position.0, dot.position.1);
        }
        context.set_line_width(5.);
        context.stroke();
    }
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    match provider.load_from_path(&"./theming/style.css") {
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
            print! {"CSS file to customize the keyboard could not be loaded. The file might be missing or broken. Using default CSS"}
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}