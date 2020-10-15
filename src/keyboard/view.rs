use std::collections::HashMap;

use crate::keyboard::{Key, KeyArrangement, KeyMeta, RESOLUTIONX, RESOLUTIONY};

#[derive(Debug)]
pub struct View {
    key_coordinates: HashMap<(i32, i32), Key>,
    cell_height: i32,
    cell_width: i32,
}

impl View {
    pub fn from(key_arrangement: &KeyArrangement, key_meta: &HashMap<String, KeyMeta>) -> View {
        let mut key_coordinates = HashMap::new();
        let cell_width = RESOLUTIONX / key_arrangement.no_columns; // Hashmaps keys can not be f64 so a lorge number instead of 1 is used to avoid large errors from rounding the result
        let cell_height = RESOLUTIONY / key_arrangement.no_rows;
        // Get the name and location and size of each key that will be in this view
        for (key_id, location) in &key_arrangement.key_arrangement {
            // Make a new key based on the key meta information
            let key = Key::from(&key_id, key_meta.get(key_id).unwrap());
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
        View {
            key_coordinates,
            cell_height,
            cell_width,
        }
    }

    pub fn get_closest_key(&self, input_x: i32, input_y: i32) -> Option<&Key> {
        let mut closest_key = None;
        let mut closest_distance = i32::MAX;
        let max_deltas = (2 * self.cell_width, 2 * self.cell_height);
        for key_coodinate in self.key_coordinates.keys() {
            let distance_new_point =
                View::get_distance(*key_coodinate, (input_x, input_y), max_deltas);
            if distance_new_point < closest_distance {
                closest_key = self.key_coordinates.get(key_coodinate);
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

    fn get_distance(point_a: (i32, i32), point_b: (i32, i32), max_delta: (i32, i32)) -> i32 {
        let delta_x = (point_a.0 - point_b.0).abs();
        let delta_y = (point_a.1 - point_b.1).abs();
        if delta_x >= max_delta.0 || delta_y >= max_delta.1 {
            i32::MAX
        } else {
            let tmp = (delta_x.pow(2) + delta_y.pow(2)) as f64;
            tmp.sqrt() as i32
        }
    }
}
