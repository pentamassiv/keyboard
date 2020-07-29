use fallback_layout::FALLBACK_LAYOUT;
use gtk::{ButtonExt, GridExt, WidgetExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path;
mod fallback_layout;

const PATH_TO_LAYOUTS: &str = "./data/keyboards";
const FALLBACK_LAYOUT_NAME: &str = "Fallback";

/// The root element describing an entire keyboard
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    views: HashMap<String, Vec<ButtonIds>>,
    outlines: HashMap<String, Outline>,
}

/// Buttons are embedded in a single string
type ButtonIds = String;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
// floats are not possible so this needs to be an integer value. These values reflect how many spaces in the grid of buttons the outline should take
pub enum Outline {
    Standard = 4,
    Half = 2,
    OneAndAHalf = 6,
    Double = 8,
    Quadruple = 16,
}

enum LayoutSource {
    YamlFile(path::PathBuf),
    FallbackStr,
}
impl Layout {
    fn from(source: LayoutSource) -> Result<(String, Layout), serde_yaml::Error> {
        let mut layout_name: String = String::from(FALLBACK_LAYOUT_NAME);
        let layout = match source {
            LayoutSource::YamlFile(path) => {
                layout_name = String::from(path.file_stem().unwrap().to_str().unwrap());
                let file_descriptor: String = format!("{}", &path.display());
                let yaml_file = File::open(&file_descriptor).expect("No file found!");
                serde_yaml::from_reader(yaml_file)
            }
            LayoutSource::FallbackStr => serde_yaml::from_str(&FALLBACK_LAYOUT),
        };

        match layout {
            Ok(layout) => Ok((layout_name, layout)),
            Err(err) => Err(err),
        }
    }
    /*pub fn get_grids_for_layout(&self, layout_name: &str) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (view_name, view) in &self.views {
            let mut button_grid = gtk::Grid::new();
            for button_id_string in view {
                button_rows.push(create_buttons_from_string(button_id_string));
            }
            result.insert(String::from(view_name), button_rows);
        }
        result
    }*/
    /*pub fn get_button_grid_for_all_views(&self) {
            let view_stack = gtk::Stack::new();
            view_stack.set_transition_type(gtk::StackTransitionType::None);
            for (view_name, view_grid) in self.views {
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
    }*/

    pub fn build_button_grid(&self) -> HashMap<String, gtk::Grid> {
        let mut result = HashMap::new();
        for (view_name, view) in &self.views {
            let grid = gtk::Grid::new();
            println!("view_name: {}", view_name);
            //grid.set_column_homogeneous(true);
            //grid.set_hexpand(true);
            //grid.set_valign(gtk::Align::Fill);
            // Get a vector that contains a vector for each row of the view. The contained vector contains the sizes of the buttons
            let mut vec_with_rows_of_buttons_and_sizes = Vec::new();
            let mut vec_row_widths = Vec::new();
            for row in view {
                let mut row_width = 0;
                let mut vec_of_buttons_with_sizes = Vec::new();
                for button_id in row.split_ascii_whitespace() {
                    let size_for_id = self.get_size_of_button(&button_id);
                    row_width += size_for_id;
                    let button = gtk::Button::with_label(button_id);
                    button.set_hexpand(true);
                    vec_of_buttons_with_sizes.push((size_for_id, button));
                }
                vec_with_rows_of_buttons_and_sizes.push(vec_of_buttons_with_sizes);
                vec_row_widths.push(row_width);
            }
            //println!("Vec of buttons and sizes:");
            //println!("{:?}", vec_with_rows_of_buttons_and_sizes);
            println!("Vec of row widths:");
            println!("{:?}", vec_row_widths);
            //Get the widest row
            let max_row_width = vec_row_widths
                .iter()
                .max()
                .expect("View needs at least one button");
            //let mut row_no = 0;
            for (row_no, row) in vec_with_rows_of_buttons_and_sizes.into_iter().enumerate() {
                let mut position = (max_row_width - vec_row_widths.get(row_no).unwrap()) / 2;
                for (size, button) in row {
                    println!(
                        "buttonlabel: {}, position: {}, row_no: {}, size: {}",
                        button.get_label().unwrap(),
                        position,
                        row_no,
                        size
                    );
                    grid.attach(&button, position, row_no as i32, size, 1);
                    position += size;
                }
                //row_no += 1;
            }
            result.insert(String::from(view_name), grid);
        }
        //println!("{:?}",result);
        result
    }

    pub fn get_size_of_button(&self, button_id: &str) -> i32 {
        self.outlines
            .get(button_id)
            .unwrap_or(&Outline::Standard)
            .to_owned() as i32
    }
}

//fn create_buttons_from_string(string_with_button_ids: &str) -> Vec<gtk::Button> {
//    let mut button_vec = Vec::new();
//    for button_id in string_with_button_ids.split_ascii_whitespace() {
//        let button = gtk::Button::with_label(button_id);
//        button_vec.push(button);
//    }
//    button_vec
//}

fn add_layout_to_hashmap(
    hashmap_with_layouts: &mut HashMap<String, Layout>,
    layout_result: Result<(String, Layout), serde_yaml::Error>,
) {
    match layout_result {
        Ok((file_name, layout)) => {
            hashmap_with_layouts.insert(file_name, layout);
        }
        Err(err) => {
            eprintln!(
                "Error loading layout. File was skipped. Error description: {}",
                err
            );
        }
    }
}

pub fn get_layouts() -> HashMap<String, Layout> {
    let mut layouts = HashMap::new();

    // Try loading layouts from directory
    if let Ok(paths) = std::fs::read_dir(PATH_TO_LAYOUTS) {
        // Load layout from all yaml files in the directory. Other files and subdirectories are ignored
        for file in paths
            .filter_map(|x| x.ok())
            .filter(|x| x.path().extension().is_some() && x.path().extension().unwrap() == "yaml")
        {
            let layout_source = LayoutSource::YamlFile(file.path());
            add_layout_to_hashmap(&mut layouts, Layout::from(layout_source));
        }
    }

    // If no layout was loaded, use hardcoded fallback layout
    if layouts.is_empty() {
        let layout_source = LayoutSource::FallbackStr;
        add_layout_to_hashmap(&mut layouts, Layout::from(layout_source));
    };
    layouts
}
