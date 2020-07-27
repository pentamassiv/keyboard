use fallback_layout::FALLBACK_LAYOUT;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

mod fallback_layout;

const PATH_TO_LAYOUTS: &str = "./data/keyboards";

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
struct Outline {
    width: f64,
    height: f64,
}

pub fn get_layouts() -> HashMap<String, Layout> {
    let mut layouts = HashMap::new();

    // Try loading layouts from directory
    if let Ok(paths) = std::fs::read_dir(PATH_TO_LAYOUTS) {
        // Load layout from all yaml files in the directory
        for file in paths
            .filter_map(|x| x.ok())
            .filter(|x| x.path().extension().is_some() && x.path().extension().unwrap() == "yaml")
        {
            let file_descriptor: String = format!("{}", &file.path().display());
            let file_name: String =
                String::from(file.path().file_stem().unwrap().to_str().unwrap());
            let yaml_file = File::open(&file_descriptor).expect("No file found!");
            let res = serde_yaml::from_reader(yaml_file);

            match res {
                Ok(res) => {
                    layouts.insert(file_name, res);
                }
                Err(err) => {
                    eprintln!(
                        "Error loading layout from file {}. File was skipped",
                        &file_descriptor
                    );
                    eprintln!("Error description: {}", err);
                }
            }
        }
    }

    // If no layout was loaded, use hardcoded fallback layout
    if layouts.is_empty() {
        let res = serde_yaml::from_str(&FALLBACK_LAYOUT);

        match res {
            Ok(res) => {
                eprintln!("Fallback layout used: {:?}", &res);
                layouts.insert(String::from("Fallback"), res);
            }
            Err(err) => {
                eprintln!("Error: Fallback failed!{}", err);
            }
        }
    };
    layouts
}
