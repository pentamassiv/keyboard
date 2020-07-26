use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

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

fn main() {
    let yaml_file = File::open("./data/keyboards/us.yaml").expect("No file found!");
    let res = serde_yaml::from_reader(yaml_file);

    if res.is_ok() {
        let layout: Layout = res.unwrap();
        println!("{:?}", layout);
    } else {
        println!("Error");
    }
}
