// Imports from other crates
use std::collections::HashMap;
use std::path;

// Imports from other modules
use super::deserialized_structs::LayoutDeserialized;
use crate::config::directories;

// Enumeration to differentiate between the source for a layout
pub enum LayoutSource {
    YamlFile(path::PathBuf),
    FallbackStr,
}

/// The LayoutYamlParser parses yaml files to search for definitions of a layout
pub struct LayoutYamlParser;
impl LayoutYamlParser {
    /// Search a directory for yaml files and parse them.
    /// If the file contains a valid definition of a layout, add it to the HashMap that will be returned
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
                        // Try deserializing a valid layout from the file
                        let layout_source = LayoutSource::YamlFile(file.path());
                        // If it is a valid layout, add it to the HashMap
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

        // If no layout for the language set with locale was loaded, use the fallback layout
        let locale_language = crate::get_locale_language();
        if !layouts.contains_key(&locale_language) {
            warn!("No yaml files describing a layout were found. Trying to create fallback layout");
            let layout_source = LayoutSource::FallbackStr;
            LayoutYamlParser::add_layout_to_hashmap(
                &mut layouts,
                LayoutDeserialized::from(layout_source),
            );
        };
        layouts
    }

    /// If a layout was deserialized, add it to the HashMap
    fn add_layout_to_hashmap(
        hashmap_with_layouts: &mut HashMap<String, LayoutDeserialized>,
        layout_result: Result<(String, LayoutDeserialized), serde_yaml::Error>,
    ) {
        match layout_result {
            Ok((file_name, layout)) => {
                hashmap_with_layouts.insert(file_name, layout);
            }
            Err(err) => {
                error!(
                    "Error loading layout. File was skipped. Error description: {}",
                    err
                );
            }
        }
    }
}
