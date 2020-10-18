// Imports from other crates
use std::path::PathBuf;

pub const CSS_FILE_REL: &str = ".fingerboard/data/theming/style.css";
pub const LAYOUT_PATH_REL: &str = ".fingerboard/data/keyboards";
pub const ICON_DIR_REL: &str = ".fingerboard/data/icons/";

/// Get the absolute path from a relative path
/// The absolute path assumes the current directory is the users HOME directory
pub fn get_absolute_path(path_in_home_dir: &str) -> Option<PathBuf> {
    // Try to get the HOME directory
    if let Some(path) = home::home_dir() {
        // Append the relative path
        let mut new_path = path;
        new_path.push(path_in_home_dir);
        info!("Path is {:?}", new_path);
        Some(new_path)
    } else {
        error!("Unable to determine the home directory");
        None
    }
}
