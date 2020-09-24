use crate::keyboard::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct View {
    key_coordinates: HashMap<(i32, i32), Key>,
}

impl View {
    pub fn from(key_arrangement: &KeyArrangement, key_meta: &HashMap<String, KeyMeta>) -> View {
        let mut key_coordinates = HashMap::new();
        let cell_width = RESOLUTIONX / key_arrangement.no_columns; // Hashmaps keys can not be f64 so a lorge number instead of 1 is used to avoid large errors from rounding the result
        let cell_height = RESOLUTIONY / key_arrangement.no_rows;
        // Get the name and location and size of each key that will be in this view
        for (key_name, location) in &key_arrangement.key_arrangement {
            // Make a new key based on the key meta information
            let key = Key::from(&key_name, key_meta.get(key_name).unwrap());
            // The keys will be arranged in a grid so if a key has a size of e.g. two,
            // a clone of the key needs to be placed in each of the two cells that the wide key would cover
            let (x, y) = (location.x, location.y); // top left edge of the "button"
            for width in 0..location.width {
                // The same is true for the height of the key
                for height in 0..location.height {
                    let (x_rel, y_rel) = (x + width, y + height);
                    // Moves the location of the key half a column to the right and bottom so that it is in the center of the buttons of the UI and not the top left corner
                    let x_rel = x_rel * cell_width + cell_width / 2;
                    let y_rel = y_rel * cell_height + cell_height / 2;
                    key_coordinates.insert((x_rel, y_rel), key.clone());
                }
            }
        }
        key_coordinates.shrink_to_fit();
        View { key_coordinates }
    }

    pub fn get_closest_key(&self, input_x: i32, input_y: i32) -> Option<&Key> {
        let mut closest_key = None;
        let mut closest_distance = i32::MAX;
        for (x, y) in self.key_coordinates.keys() {
            let distance_new_point = self.get_distance(*x, *y, input_x, input_y);
            if distance_new_point < closest_distance {
                closest_key = self.key_coordinates.get(&(*x, *y));
                closest_distance = distance_new_point;
            }
        }
        let mut result = None;
        if let Some(key) = closest_key {
            let keys = key;
            result = Some(keys);
        }
        result
    }

    fn get_distance(&self, point_a_x: i32, point_a_y: i32, point_b_x: i32, point_b_y: i32) -> i32 {
        let delta_x = (point_a_x - point_b_x).abs();
        let delta_y = (point_a_y - point_b_y).abs();
        let tmp = (delta_x.pow(2) + delta_y.pow(2)) as f64;
        tmp.sqrt() as i32
    }
}
