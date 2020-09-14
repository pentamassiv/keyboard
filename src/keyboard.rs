use super::submitter::*;
use crate::user_interface::MessagePipe;
use gtk::*;
use gtk::{ButtonExt, GridExt, StyleContextExt, WidgetExt};

use std::collections::HashMap;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

pub mod parser;
pub use parser::{KeyAction, KeyEvent}; // Re-export
pub mod vk_sub_connector;
pub mod vk_ui_connector;

pub const ICON_FOLDER: &str = "./data/icons/";
pub const RESOLUTIONX: i32 = 10000;
pub const RESOLUTIONY: i32 = 10000;

pub const KEYBOARD_DEFAULT_LAYOUT: &str = "us";
pub const KEYBOARD_DEFAULT_VIEW: &str = "base";

#[derive(Debug, Clone)]
pub struct Key {
    actions: HashMap<KeyEvent, Vec<KeyAction>>,
    button: gtk::Button,
    popover: gtk::Popover,
}
impl Key {
    fn from(
        relm: &relm::Relm<crate::user_interface::Win>,
        key_id: &str,
        key_meta: Option<&parser::KeyMeta>,
    ) -> Key {
        let button = gtk::Button::new();
        button.set_label(key_id);
        button.set_hexpand(true);
        button.get_style_context().add_class("key");
        let popover = gtk::Popover::new(Some(&button));
        let mut actions = Self::make_default_actions(key_id);
        if let Some(key_meta) = key_meta {
            if let Some(style_classes) = &key_meta.styles {
                for style_classes in style_classes {
                    button.get_style_context().add_class(style_classes);
                }
            }
            if let Some(key_display_enum) = &key_meta.key_display {
                match key_display_enum {
                    parser::KeyDisplay::Text(label_text) => button.set_label(&label_text),
                    parser::KeyDisplay::Image(icon_name) => {
                        let mut icon_path = String::from(ICON_FOLDER);
                        icon_path.push_str(icon_name);
                        let image = gtk::Image::from_file(&icon_path);
                        button.set_image(Some(&image));
                        button.set_always_show_image(true);
                        button.set_label("");
                    }
                }
            }
            if let Some(key_actions) = &key_meta.actions {
                actions = key_actions.clone();
            }
            if let Some(popup) = &key_meta.popup {
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                for popup_string in popup {
                    let new_popup_button = gtk::Button::new();
                    new_popup_button.set_label(popup_string);
                    hbox.add(&new_popup_button);
                    let tmp_popover_ref = popover.clone();
                    new_popup_button.connect_clicked(move |_| tmp_popover_ref.hide());
                    /*relm::connect!(
                        relm,
                        new_popup_button,
                        connect_button_release_event(clicked_button, _),
                        return (
                            Some(crate::user_interface::Msg::EnterString(
                                clicked_button.get_label().unwrap().to_string(),
                                false,
                            )),
                            gtk::Inhibit(false)
                        )
                    );*/
                }
                popover.add(&hbox);
            }
        }
        Key {
            actions,
            button,
            popover,
        }
    }

    fn make_default_actions(key_id: &str) -> HashMap<KeyEvent, Vec<KeyAction>> {
        let mut actions = HashMap::new();
        let key_event = KeyEvent::ShortPress;
        let action_events = KeyAction::EnterString(key_id.to_string());
        actions.insert(key_event, vec![action_events]);
        actions
    }

    pub fn activate(&self, win: &crate::user_interface::Win, key_event: &KeyEvent) {
        let tmp_vec = Vec::new();
        let actions_vec = self.actions.get(&key_event).unwrap_or(&tmp_vec);
        for action in actions_vec {
            match action {
                KeyAction::EnterKeycode(keycode) => {
                    win.relm.stream().emit(crate::user_interface::Msg::Submit(
                        Submission::Keycode(keycode.to_string()),
                    ));
                    //self.submitter.submit(Submission::Keycode(keycode))
                }
                KeyAction::EnterString(text) => {
                    win.relm
                        .stream()
                        .emit(crate::user_interface::Msg::Submit(Submission::Text(
                            text.to_string(),
                        )));
                    //self.submitter.submit(Submission::Keycode(keycode))
                }
                KeyAction::SwitchView(new_view) => {
                    win.relm
                        .stream()
                        .emit(crate::user_interface::Msg::SwitchView(new_view.to_string()));
                }
                KeyAction::Modifier(modifier) => {
                    win.relm.stream().emit(crate::user_interface::Msg::Submit(
                        Submission::Keycode("SHIFT".to_string()), // TODO: set up properly
                    ));
                }
                KeyAction::Erase => {
                    win.relm
                        .stream()
                        .emit(crate::user_interface::Msg::Submit(Submission::Erase));
                }
                KeyAction::OpenPopup => {
                    self.popover.show_all();
                }
            }
        }
        //self.button.activate(); // Disabled, because the transition takes too long and makes it looks sluggish
    }
}

pub enum UIMsg {
    Visable(bool),
    HintPurpose(ContentHint, ContentPurpose),
    SwitchView(String),
    SwitchLayout(String),
}

pub trait EmitUIMsg {
    fn emit(&self, message: UIMsg);
}

//#[derive(Debug)]
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    pub active_view: (String, String),
    active_keys: Vec<Key>,
    submitter: Submitter<vk_sub_connector::SubConnector>,
}

impl Keyboard {
    pub fn new(ui_message_pipe: MessagePipe) -> Keyboard {
        let ui_connector = vk_ui_connector::UIConnector::new(ui_message_pipe);
        let sub_connector = vk_sub_connector::SubConnector::new(ui_connector);
        let submitter = Submitter::new(sub_connector);
        Keyboard {
            views: HashMap::new(),
            active_view: (
                String::from(KEYBOARD_DEFAULT_LAYOUT),
                String::from(KEYBOARD_DEFAULT_VIEW),
            ),
            active_keys: Vec::new(),
            submitter,
        }
    }
    pub fn fetch_events(&mut self) {
        self.submitter.fetch_events();
    }

    pub fn submit(&mut self, submission: Submission) {
        println!("Submit: {:?}", submission);
        self.submitter.submit(submission);
    }

    pub fn init(
        &mut self,
        relm: &relm::Relm<crate::user_interface::Win>,
        layout_metas: HashMap<String, parser::LayoutMeta>,
    ) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (layout_name, layout_meta) in layout_metas {
            for (view_name, grid) in self.add_layout(relm, &layout_name, layout_meta) {
                let grid_name = Self::make_view_name(&layout_name, &view_name);
                result.insert(grid_name, grid);
            }
        }
        result
    }

    pub fn get_view_name(&self) -> (String, String) {
        (self.active_view.0.clone(), self.active_view.1.clone())
    }
    pub fn make_view_name(layout_name: &str, view_name: &str) -> String {
        let mut layout_view_name = String::from(layout_name);
        layout_view_name.push('_'); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }

    fn add_layout(
        &mut self,
        relm: &relm::Relm<crate::user_interface::Win>,
        layout_name: &str,
        layout_meta: parser::LayoutMeta,
    ) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (view_name, view_meta) in &layout_meta.views {
            let grid = gtk::Grid::new();
            grid.set_column_homogeneous(true);
            grid.set_row_homogeneous(true);
            let button_sizes = self.get_all_button_sizes(view_meta, &layout_meta);
            let mut row_widths: Vec<i32> = Vec::new();
            for button_row in &button_sizes {
                row_widths.push(button_row.iter().sum());
            }
            let max_row_width: i32 = *row_widths
                .iter()
                .max()
                .expect("View needs at least one button");
            let width_of_cell = RESOLUTIONX / max_row_width;
            let height_of_cell = RESOLUTIONY / (row_widths.len() as i32);
            let half_width_of_cell = width_of_cell / 2;
            let half_height_of_cell = height_of_cell / 2;
            let mut view = super::keyboard::View::new();
            for (row_no, row) in button_sizes.into_iter().enumerate() {
                let mut position = (max_row_width - row_widths[row_no]) / 2;
                let mut button_id_iter = view_meta[row_no].split_ascii_whitespace();
                for size in row {
                    if let Some(key_id) = button_id_iter.next() {
                        let key = Key::from(relm, key_id, layout_meta.buttons.get(key_id));
                        grid.attach(&key.button, position, row_no as i32, size, 1);
                        for s in 0..size {
                            view.add_key_coordinate(
                                (position + s) * width_of_cell + half_width_of_cell,
                                (row_no as i32) * height_of_cell + half_height_of_cell,
                                key.clone(),
                            )
                        }
                        position += size;
                    }
                }
            }
            result.insert(String::from(view_name), grid);
            self.views
                .insert((String::from(layout_name), String::from(view_name)), view);
        }
        result
    }

    pub fn get_closest_key(
        &self,
        layout_name: &str,
        view_name: &str,
        x: i32,
        y: i32,
    ) -> Option<&Key> {
        if let Some(view) = self
            .views
            .get(&(layout_name.to_string(), view_name.to_string()))
        {
            view.get_closest_key(x, y)
        } else {
            None
        }
    }
    fn get_all_button_sizes(
        &self,
        button_ids: &[String],
        layout_meta: &parser::LayoutMeta,
    ) -> Vec<Vec<i32>> {
        let mut button_sizes = Vec::new();
        for row in button_ids {
            let mut row_vec = Vec::new();
            for button_id in row.split_ascii_whitespace() {
                let size_for_id = layout_meta.get_size_of_button(&button_id);
                row_vec.push(size_for_id);
            }
            button_sizes.push(row_vec);
        }
        button_sizes
    }
}

#[derive(Debug)]
pub struct View {
    key_coordinates: HashMap<(i32, i32), Key>,
}

impl View {
    pub fn new() -> View {
        View {
            key_coordinates: HashMap::new(),
        }
    }

    pub fn add_key_coordinate(&mut self, x: i32, y: i32, key: Key) {
        self.key_coordinates.insert((x, y), key);
    }

    fn get_closest_key(&self, input_x: i32, input_y: i32) -> Option<&Key> {
        let mut closest_key = None;
        let mut closest_distance = i32::MAX;
        for (x, y) in self.key_coordinates.keys() {
            let distance_new_point = self.get_distance((*x, *y), (input_x, input_y));
            if distance_new_point < closest_distance {
                closest_key = self.key_coordinates.get(&(*x, *y));
                closest_distance = distance_new_point;
            }
        }
        let mut result = None;
        if let Some(key) = closest_key {
            let keys = key;
            result = Some(keys);
        }
        result
    }

    fn get_distance(&self, coordinate_a: (i32, i32), coordinate_b: (i32, i32)) -> i32 {
        let delta_x = (coordinate_a.0 - coordinate_b.0).abs();
        let delta_y = (coordinate_a.1 - coordinate_b.1).abs();
        let tmp = (delta_x.pow(2) + delta_y.pow(2)) as f64;
        tmp.sqrt() as i32
    }
}
