use crate::config::directories;
use crate::config::fallback_layout::FALLBACK_LAYOUT;
use crate::user_interface;
use gtk::{ButtonExt, GridExt, StyleContextExt, WidgetExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path;

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
// These values reflect how many spaces in the grid of buttons the outline should take. That's why it needs to be an integer value
enum Outline {
    Standard = 2,
    Half = 1,
    OneAndAHalf = 3,
    Double = 4,
    Quadruple = 8,
}

enum LayoutSource {
    YamlFile(path::PathBuf),
    FallbackStr,
}
impl Layout {
    fn from(source: LayoutSource) -> Result<(String, Layout), serde_yaml::Error> {
        let mut layout_name: String = String::from(directories::FALLBACK_LAYOUT_NAME);
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
    // Returns a grid with all the buttons and a tupel with the grids number of rows and columns
    pub fn build_button_grid_and_its_dimensions(
        &self,
        relm: &relm::Relm<user_interface::Win>,
    ) -> HashMap<String, (gtk::Grid, (usize, usize))> {
        let mut result = HashMap::new();
        for (view_name, view) in &self.views {
            let grid = gtk::Grid::new();
            grid.set_column_homogeneous(true);
            grid.set_row_homogeneous(true);
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
                    //button.set_size_request(1,2);
                    button.set_hexpand(true);
                    button.get_style_context().add_class("key");
                    relm::connect!(
                        relm,
                        button,
                        connect_clicked(clicked_button),
                        user_interface::Msg::KeyPress(
                            clicked_button.get_label().unwrap().to_string()
                        )
                    );
                    vec_of_buttons_with_sizes.push((size_for_id, button));
                }
                vec_with_rows_of_buttons_and_sizes.push(vec_of_buttons_with_sizes);
                vec_row_widths.push(row_width);
            }
            //Get the widest row
            let max_row_width = *vec_row_widths
                .iter()
                .max()
                .expect("View needs at least one button");
            let mut max_row_heigth = 0;
            for (row_no, row) in vec_with_rows_of_buttons_and_sizes.into_iter().enumerate() {
                let mut position = (max_row_width - vec_row_widths.get(row_no).unwrap()) / 2;
                for (size, button) in row {
                    grid.attach(&button, position, row_no as i32, size, 1);
                    position += size;
                }
                max_row_heigth = row_no + 1;
            }
            result.insert(
                String::from(view_name),
                (
                    grid,
                    (max_row_width.to_owned() as usize, max_row_heigth.to_owned()),
                ),
            );
        }
        result
    }

    fn get_size_of_button(&self, button_id: &str) -> i32 {
        self.outlines
            .get(button_id)
            .unwrap_or(&Outline::Standard)
            .to_owned() as i32
    }
}

pub struct LayoutParser;
impl LayoutParser {
    pub fn get_layouts() -> HashMap<String, Layout> {
        let mut layouts = HashMap::new();

        // Try loading layouts from directory
        if let Ok(paths) = std::fs::read_dir(directories::PATH_TO_LAYOUTS) {
            // Load layout from all yaml files in the directory. Other files and subdirectories are ignored
            for file in paths.filter_map(|x| x.ok()).filter(|x| {
                x.path().extension().is_some() && x.path().extension().unwrap() == "yaml"
            }) {
                let layout_source = LayoutSource::YamlFile(file.path());
                LayoutParser::add_layout_to_hashmap(&mut layouts, Layout::from(layout_source));
            }
        }

        // If no layout was loaded, use hardcoded fallback layout
        if layouts.is_empty() {
            let layout_source = LayoutSource::FallbackStr;
            LayoutParser::add_layout_to_hashmap(&mut layouts, Layout::from(layout_source));
        };
        layouts
    }
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
}
