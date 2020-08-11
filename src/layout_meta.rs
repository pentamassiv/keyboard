use crate::config::directories;
use crate::config::fallback_layout::FALLBACK_LAYOUT;
use crate::user_interface;
use gtk::{ButtonExt, GridExt, StyleContextExt, WidgetExt};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::path;

/// The root element describing an entire keyboard
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LayoutMeta {
    pub views: HashMap<String, Vec<ButtonIds>>,
    pub buttons: HashMap<String, crate::keyboard::KeyMeta>,
}

/// Buttons are embedded in a single string
type ButtonIds = String;

// These values reflect how many spaces in the grid of buttons the outline should take. That's why it needs to be an integer value
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Outline {
    #[serde(rename = "standard")]
    Standard = 2,
    #[serde(rename = "half")]
    Half = 1,
    #[serde(rename = "one_and_a_half")]
    OneAndAHalf = 3,
    #[serde(rename = "double")]
    Double = 4,
    #[serde(rename = "quadruple")]
    Quadruple = 8,
}

enum LayoutMetaSource {
    YamlFile(path::PathBuf),
    FallbackStr,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
enum Action {
    #[serde(rename = "locking")]
    Locking {
        lock_view: String,
        unlock_view: String,
    },
    #[serde(rename = "set_view")]
    SetView(String),
    #[serde(rename = "show_prefs")]
    ShowPrefs,
    #[serde(rename = "erase")]
    Erase,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
enum Modifier {
    Control,
    Shift,
    Lock,
    #[serde(alias = "Mod1")]
    Alt,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

impl LayoutMeta {
    fn from(source: LayoutMetaSource) -> Result<(String, LayoutMeta), serde_yaml::Error> {
        let mut layout_name: String = String::from(directories::FALLBACK_LAYOUT_NAME);
        let layout = match source {
            LayoutMetaSource::YamlFile(path) => {
                layout_name = String::from(path.file_stem().unwrap().to_str().unwrap());
                let file_descriptor: String = format!("{}", &path.display());
                let yaml_file = File::open(&file_descriptor).expect("No file found!");
                serde_yaml::from_reader(yaml_file)
            }
            LayoutMetaSource::FallbackStr => serde_yaml::from_str(&FALLBACK_LAYOUT),
        };

        match layout {
            Ok(layout) => Ok((layout_name, layout)),
            Err(err) => Err(err),
        }
    }

    pub fn get_size_of_button(&self, button_id: &str) -> i32 {
        if let Some(key) = self.buttons.get(button_id) {
            if let Some(outline) = key.outline {
                return outline as i32;
            }
        }
        Outline::Standard as i32
    }
}

pub struct LayoutYamlParser;
impl LayoutYamlParser {
    pub fn get_layouts() -> HashMap<String, LayoutMeta> {
        let mut layouts = HashMap::new();

        // Try loading layouts from directory
        if let Ok(paths) = std::fs::read_dir(directories::PATH_TO_LAYOUTS) {
            // Load layout from all yaml files in the directory. Other files and subdirectories are ignored
            for file in paths.filter_map(|x| x.ok()).filter(|x| {
                x.path().extension().is_some() && x.path().extension().unwrap() == "yaml"
            }) {
                let layout_source = LayoutMetaSource::YamlFile(file.path());
                LayoutYamlParser::add_layout_to_hashmap(
                    &mut layouts,
                    LayoutMeta::from(layout_source),
                );
            }
        }

        // If no layout was loaded, use hardcoded fallback layout
        if layouts.is_empty() {
            let layout_source = LayoutMetaSource::FallbackStr;
            LayoutYamlParser::add_layout_to_hashmap(&mut layouts, LayoutMeta::from(layout_source));
        };
        layouts
    }

    fn add_layout_to_hashmap(
        hashmap_with_layouts: &mut HashMap<String, LayoutMeta>,
        layout_result: Result<(String, LayoutMeta), serde_yaml::Error>,
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
