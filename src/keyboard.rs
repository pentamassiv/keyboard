use crate::layout_meta::Outline;
use gtk::{ButtonExt, GridExt, StyleContextExt, WidgetExt};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

pub const ICON_FOLDER: &str = "./data/icons/";
pub const RESOLUTIONX: i32 = 10000;
pub const RESOLUTIONY: i32 = 10000;

pub const KEYBOARD_DEFAULT_LAYOUT: &str = "us";
pub const KEYBOARD_DEFAULT_VIEW: &str = "base";

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct KeyMeta {
    actions: Option<HashMap<KeyEvent, Vec<KeyAction>>>,
    key_display: Option<KeyDisplay>,
    pub outline: Option<Outline>,
    popup: Option<Vec<KeyMeta>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
enum KeyEvent {
    #[serde(rename = "long_press")]
    LongPress,
    #[serde(rename = "short_press")]
    ShortPress,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
enum KeyAction {
    #[serde(rename = "modifier")]
    Modifier(String),
    #[serde(rename = "switch_view")]
    SwitchView(String),
    #[serde(rename = "erase")]
    Erase,
    #[serde(rename = "enter_keycode")]
    EnterKeycode(Vec<String>),
    #[serde(rename = "open_popup")]
    OpenPopup,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
enum KeyDisplay {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image(String),
}

#[derive(Debug)]
struct Key {
    actions: HashMap<KeyEvent, Vec<KeyAction>>,
    button: gtk::Button,
}
impl Key {
    fn from(key_id: &str, key_meta: Option<&KeyMeta>) -> Key {
        let button = gtk::Button::new();
        button.set_label(key_id);
        button.set_hexpand(true);
        button.get_style_context().add_class("key");
        let mut actions = Self::make_default_actions(key_id);
        if let Some(key_meta) = key_meta {
            if let Some(key_display_enum) = &key_meta.key_display {
                match key_display_enum {
                    KeyDisplay::Text(label_text) => button.set_label(&label_text),
                    KeyDisplay::Image(icon_name) => {
                        let mut icon_path = String::from(ICON_FOLDER);
                        icon_path.push_str(icon_name);
                        println!("resource_path: {}", icon_path);
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
        }
        Key { actions, button }
    }

    fn make_default_actions(key_id: &str) -> HashMap<KeyEvent, Vec<KeyAction>> {
        let mut actions = HashMap::new();
        let key_event = KeyEvent::ShortPress;
        let action_events = KeyAction::EnterKeycode(vec![key_id.to_string()]);
        actions.insert(key_event, vec![action_events]);
        actions
    }
}

#[derive(Debug)]
pub struct Keyboard {
    pub views: HashMap<(String, String), View>,
    keys: HashMap<(String, String, String), Key>, //Key for HashMap is (layout_name, key_name)
    active_view: (String, String),
    active_keys: Vec<Key>,
}
impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            views: HashMap::new(),
            active_view: (
                String::from(KEYBOARD_DEFAULT_LAYOUT),
                String::from(KEYBOARD_DEFAULT_VIEW),
            ),
            keys: HashMap::new(),
            active_keys: Vec::new(),
        }
    }

    pub fn init(
        &mut self,
        layout_metas: HashMap<String, crate::layout_meta::LayoutMeta>,
    ) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (layout_name, layout_meta) in layout_metas {
            for (view_name, grid) in self.add_layout(&layout_name, layout_meta) {
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
        layout_view_name.push_str("_"); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }

    fn add_layout(
        &mut self,
        layout_name: &str,
        layout_meta: crate::layout_meta::LayoutMeta,
    ) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (view_name, view_meta) in &layout_meta.views {
            self.add_keys(layout_name, view_name, view_meta, &layout_meta.buttons);
            let grid = gtk::Grid::new();
            grid.set_column_homogeneous(true);
            grid.set_row_homogeneous(true);
            // Get a vector that contains a vector for each row of the view_meta. The contained vector contains the sizes of the buttons
            //Get the widest row

            /*let button = gtk::Button::with_label(button_id);
            button.set_hexpand(true);
            button.get_style_context().add_class("key");
            relm::connect!(
                relm,
                button,
                connect_button_release_event(clicked_button, _),
                return (
                    Some(crate::user_interface::Msg::EnterInput(
                        clicked_button.get_label().unwrap().to_string(),
                        false
                    )),
                    gtk::Inhibit(false)
                )
            );*/
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
                        let key_option = self.keys.get(&(
                            layout_name.to_string(),
                            view_name.to_string(),
                            key_id.to_string(),
                        ));
                        if let Some(key) = key_option {
                            grid.attach(&key.button, position, row_no as i32, size, 1);
                            for s in 0..size {
                                view.add_button_coordinate(
                                    (position + s) * width_of_cell + half_width_of_cell,
                                    (row_no as i32) * height_of_cell + half_height_of_cell,
                                    key.button.clone(),
                                )
                            }
                            position += size;
                        }
                    }
                }
            }
            result.insert(String::from(view_name), grid);
            self.add_view(layout_name, &view_name, view);
        }
        result
    }

    pub fn add_view(&mut self, layout_name: &str, view_name: &str, view: View) {
        self.views
            .insert((String::from(layout_name), String::from(view_name)), view);
    }

    fn add_keys(
        &mut self,
        layout_name: &str,
        view_name: &str,
        view_meta: &[String],
        key_meta_hashmap: &HashMap<String, KeyMeta>,
    ) {
        for button_id_string in view_meta {
            for button_id in button_id_string.split_ascii_whitespace() {
                self.keys
                    .entry((
                        layout_name.to_string(),
                        view_name.to_string(),
                        button_id.to_string(),
                    ))
                    .or_insert_with(|| Key::from(button_id, key_meta_hashmap.get(button_id)));
            }
        }
    }

    pub fn get_closest_button(
        &self,
        layout_name: &str,
        view_name: &str,
        x: i32,
        y: i32,
    ) -> Option<gtk::Button> {
        if let Some(spacial_model_view) = self
            .views
            .get(&(layout_name.to_string(), view_name.to_string()))
        {
            spacial_model_view.get_closest_button(x, y)
        } else {
            None
        }
    }
    fn get_all_button_sizes(
        &self,
        button_ids: &[String],
        layout_meta: &crate::layout_meta::LayoutMeta,
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
    button_coordinates: HashMap<(i32, i32), gtk::Button>,
}

impl View {
    pub fn new() -> View {
        View {
            button_coordinates: HashMap::new(),
        }
    }

    pub fn add_button_coordinate(&mut self, x: i32, y: i32, button: gtk::Button) {
        self.button_coordinates.insert((x, y), button);
    }

    fn get_closest_button(&self, input_x: i32, input_y: i32) -> Option<gtk::Button> {
        let mut closest_button = None;
        let mut closest_distance = i32::MAX;
        for (x, y) in self.button_coordinates.keys() {
            let distance_new_point = self.get_distance((*x, *y), (input_x, input_y));
            if distance_new_point < closest_distance {
                closest_button = self.button_coordinates.get(&(*x, *y));
                closest_distance = distance_new_point;
            }
        }
        let mut result = None;
        if let Some(button) = closest_button {
            let buttons = button.clone();
            result = Some(buttons);
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
