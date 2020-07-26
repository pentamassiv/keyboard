use fallback_layout::FALLBACK_LAYOUT;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

mod fallback_layout;

const PATH_TO_LAYOUTS: &str = "../data/keyboards";

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

    // Try loading layouts from file
    if let Ok(path) = std::fs::read_dir(PATH_TO_LAYOUTS) {
        for file in path {
            let filepath: String = format!("{}", &file.unwrap().path().display());
            println!("filepath: {}", &filepath);
            let yaml_file = File::open(&filepath).expect("No file found!");
            let res = serde_yaml::from_reader(yaml_file);

            match res {
                Ok(res) => {
                    println!("{:?}", &res);
                    layouts.insert(filepath, res);
                }
                Err(_) => {
                    println!("Error");
                }
            }
        }
    }

    // If no layout was loaded, use hardcoded fallback layout
    if layouts.is_empty() {
        let res = serde_yaml::from_str(&FALLBACK_LAYOUT);

        match res {
            Ok(res) => {
                println!("Fallback layout used: {:?}", &res);
                layouts.insert(String::from("Fallback"), res);
            }
            Err(err) => {
                println!("Error: Fallback failed!{}", err);
            }
        }
    };
    layouts
}
