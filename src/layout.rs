use fallback_layout::FALLBACK_LAYOUT;
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
    pub fn get_buttons(&self) -> HashMap<String, Vec<Vec<gtk::Button>>> {
        let mut result = HashMap::new();
        for (view_name, view) in &self.views {
            let mut button_rows = Vec::new();
            for button_id_string in view {
                button_rows.push(create_buttons_from_string(button_id_string));
            }
            result.insert(String::from(view_name), button_rows);
        }
        result
    }
    pub fn get_size_of_button(&self, button_id: &str) -> usize {
        self.outlines
            .get(button_id)
            .unwrap_or(&Outline::Standard)
            .to_owned() as usize
    }
}

fn create_buttons_from_string(string_with_button_ids: &str) -> Vec<gtk::Button> {
    let mut button_vec = Vec::new();
    for button_id in string_with_button_ids.split_ascii_whitespace() {
        let button = gtk::Button::with_label(button_id);
        button_vec.push(button);
    }
    button_vec
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
