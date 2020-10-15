use std::path::PathBuf;

pub const CSS_FILE_REL: &str = ".fingerboard/theming/style.css";
pub const LAYOUT_PATH_REL: &str = ".fingerboard/data/keyboards";
pub const ICON_DIR_REL: &str = ".fingerboard/data/icons/";

pub fn get_absolute_path(path_in_home_dir: &str) -> Option<PathBuf> {
    if let Some(path) = home::home_dir() {
        let mut new_path = path;
        new_path.push(path_in_home_dir);
        info!("Path is {:?}", new_path);
        Some(new_path)
    } else {
        error!("Unable to determine the home directory");
        None
    }
}
