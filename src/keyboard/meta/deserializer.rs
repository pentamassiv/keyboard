use super::deserialized_structs::*;
use crate::config::directories;
use std::collections::HashMap;
use std::path;

pub enum LayoutSource {
    YamlFile(path::PathBuf),
    FallbackStr,
}

pub struct LayoutYamlParser;
impl LayoutYamlParser {
    pub fn get_layouts() -> HashMap<String, LayoutDeserialized> {
        let mut layouts = HashMap::new();

        // Try loading layouts from directory
        if let Ok(paths) = std::fs::read_dir(directories::PATH_TO_LAYOUTS) {
            // Load layout from all yaml files in the directory. Other files and subdirectories are ignored
            for file in paths.filter_map(|x| x.ok()).filter(|x| {
                x.path().extension().is_some() && x.path().extension().unwrap() == "yaml"
            }) {
                let layout_source = LayoutSource::YamlFile(file.path());
                LayoutYamlParser::add_layout_to_hashmap(
                    &mut layouts,
                    LayoutDeserialized::from(layout_source),
                );
            }
        }

        // If no layout was loaded, use hardcoded fallback layout
        if layouts.is_empty() {
            let layout_source = LayoutSource::FallbackStr;
            LayoutYamlParser::add_layout_to_hashmap(
                &mut layouts,
                LayoutDeserialized::from(layout_source),
            );
        };
        layouts
    }

    fn add_layout_to_hashmap(
        hashmap_with_layouts: &mut HashMap<String, LayoutDeserialized>,
        layout_result: Result<(String, LayoutDeserialized), serde_yaml::Error>,
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
