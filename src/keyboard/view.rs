// Imports from other crates
use std::collections::HashMap;

// Imports from other modules
use crate::keyboard::{Key, KeyArrangement, KeyMeta, RESOLUTIONX, RESOLUTIONY};

#[derive(Debug)]
/// The view contains all its keys and their location in a two-dimensional space.
/// The dimensions of this space are independent from the UI to avoid recalculation of all locations when the UI changes its size.
/// It is expected the keys get arranged in a grid. If a key spans two or more of the cells of the grid, the key needs to be cloned and set to occupy each of these cells
pub struct View {
    key_coordinates: HashMap<(i32, i32), Key>,
    cell_height: i32, // Stores the height of the cells
    cell_width: i32,  // Store the width of the cells
}

impl View {
    // Build a 'View' from the meta infos about the keys and their arrangement
    pub fn from(key_arrangement: &KeyArrangement, key_meta: &HashMap<String, KeyMeta>) -> View {
        let mut key_coordinates = HashMap::new();
        // A HashMap can't contain keys that are f64 so instead of dividing 1 by the number of columns/width, a lorge number is used and then rounded to calculate the
        // width/height of the cells
        let cell_width = RESOLUTIONX / key_arrangement.get_no_columns();
        let cell_height = RESOLUTIONY / key_arrangement.get_no_rows();
        // Get the name and location and size of each key that will be in this view
        for (key_id, location) in key_arrangement.get_key_arrangement() {
            // Make a new key based on the key meta information
            let key = Key::from(&key_id, key_meta.get(key_id).unwrap());
            // This is the location of the top left edge of the "button"
            let (x, y) = (location.x, location.y);
            // The keys will be arranged in a grid so if a key has a size of e.g. two,
            // a clone of the key needs to be placed in each of the two cells that the wide key would cover
            // The same is true for the height of the key
            for width in 0..location.width {
                for height in 0..location.height {
                    let (x_rel, y_rel) = (x + width, y + height);
                    // Moves the location of the key half a column to the right and bottom so that it is in the center of the buttons of the UI and not the top left corner
                    let x_rel = x_rel * cell_width + cell_width / 2;
                    let y_rel = y_rel * cell_height + cell_height / 2;
                    key_coordinates.insert((x_rel, y_rel), key.clone());
                }
            }
        }
        // Shrinks the HashMap to save memory
        key_coordinates.shrink_to_fit();
        View {
            key_coordinates,
            cell_height,
            cell_width,
        }
    }

    /// Gets the closest key to the coordinates of the interaction
    /// If every key is too far away, 'None' is returned
    pub fn get_closest_key(&self, input_x: i32, input_y: i32) -> Option<&Key> {
        // Set the closest key to 'None' and the closest distance to the maximum
        let mut closest_key = None;
        let mut closest_distance = i32::MAX;

        // The maximum distance a key can have to be considered as closer is twice the cell height/width to avoid very far away buttons to get pressed
        let max_deltas = (2 * self.cell_width, 2 * self.cell_height);
        // Calculate the distance between the user interaction and each of the keys
        // If the distance exceeds the maximum delta in one of the dimensions, the distance is maximum so the button can never be closer
        for key_coodinate in self.key_coordinates.keys() {
            let distance_new_point =
                View::get_distance(*key_coodinate, (input_x, input_y), max_deltas);
            // If the distance is closer than the current distance, the key is the new closest_key
            if distance_new_point < closest_distance {
                closest_key = self.key_coordinates.get(key_coodinate);
                closest_distance = distance_new_point;
            }
        }
        closest_key
    }

    /// Calculate the distance between point A and point B
    // If the distance exceeds the maximum delta in one of the dimensions, the distance is maximum
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
