use std::path::PathBuf;

pub const CSS_FILE_REL: &str = "./.fingerboard/theming/style.css";
pub const LAYOUT_PATH_REL: &str = "./.fingerboard/data/keyboards";

pub fn get_absolute_path(path_in_home_dir: &str) -> Option<PathBuf> {
    match home::home_dir() {
        Some(path) => {
            let mut new_path = path;
            new_path.push(path_in_home_dir);
            Some(new_path)
        }
        None => None,
    }
}
