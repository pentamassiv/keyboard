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
        if let Some(layout_dir_abs) = directories::get_absolute_path(directories::LAYOUT_PATH_REL) {
            info!(
                "Try searching for layout descriptions in directory {:?}",
                layout_dir_abs
            );
            match std::fs::read_dir(layout_dir_abs) {
                Ok(paths) => {
                    info!("Searching for layout description in files {:?}", paths);
                    // Load layout from all yaml files in the directory. Other files and subdirectories are ignored
                    for file in paths.filter_map(std::result::Result::ok).filter(|x| {
                        x.path().extension().is_some() && x.path().extension().unwrap() == "yaml"
                    }) {
                        info!("Searching for layout description in file {:?}", file.path());
                        let layout_source = LayoutSource::YamlFile(file.path());
                        LayoutYamlParser::add_layout_to_hashmap(
                            &mut layouts,
                            LayoutDeserialized::from(layout_source),
                        );
                    }
                }
                Err(err) => {
                    error!("There was an error when reading the directory: {}", err);
                }
            }
        }

        // If no layout was loaded, use hardcoded fallback layout
        if layouts.is_empty() {
            warn!("No yaml files describing a layout were found. Trying to create fallback layout");
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
