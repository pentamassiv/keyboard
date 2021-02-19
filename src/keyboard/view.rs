// Imports from other crates
use std::collections::HashMap;

// Imports from other modules
use crate::keyboard::{Key, KeyArrangement, KeyMeta};

#[derive(Debug)]
/// The view contains all its keys and their location in a two-dimensional space.
/// The dimensions of this space are independent from the UI to avoid recalculation of all locations when the UI changes its size.
/// It is expected the keys get arranged in a grid. If a key spans two or more of the cells of the grid, the key needs to be cloned and set to occupy each of these cells
/// Keyboards can have different dimensions. We assume all keys to be as wide as they are high
pub struct View {
    pub key_coordinates: Vec<((f64, f64), Key)>,
    cell_radius: f64, // Store the width of the cells
    row_to_column_ratio: f64,
}

impl View {
    // Build a 'View' from the meta infos about the keys and their arrangement
    pub fn from(key_arrangement: &KeyArrangement, key_meta: &HashMap<String, KeyMeta>) -> View {
        let mut key_coordinates = Vec::new();
        // A HashMap can't contain keys that are f64 so instead of dividing 1 by the number of columns/width, a lorge number is used and then rounded to calculate the
        // width/height of the cells

        let cell_radius = 1.0 / key_arrangement.get_no_columns() as f64;

        // Calculate the ratio between the rows and the columns
        // We multiply it by the factor 2 because the standard width of keys is 2 but their height is only 1
        // That was done to arrange all keys in a grid and yet allow rows with one less key to be centered
        // This is not necessary because buttons are always on the same height
        let row_to_column_ratio =
            2.0 * key_arrangement.get_no_rows() as f64 / key_arrangement.get_no_columns() as f64;

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
                    let x_rel = x_rel as f64 * cell_radius + cell_radius / 2.0;
                    let y_rel = y_rel as f64 * cell_radius + cell_radius / 2.0;
                    key_coordinates.push(((x_rel, y_rel), key.clone()));
                }
            }
        }

        // Shrinks the HashMap to save memory
        key_coordinates.shrink_to_fit();
        View {
            key_coordinates,
            cell_radius,
            row_to_column_ratio,
        }
    }

    /// Gets the closest key to the coordinates of the interaction
    /// If every key is too far away, 'None' is returned
    pub fn get_closest_key(&self, input_x: f64, input_y: f64) -> Option<&Key> {
        // Set the closest key to 'None' and the closest distance to the maximum
        let mut closest_key = None;
        let mut closest_distance = f64::MAX;

        println!("Input coordinate ({}/{})", input_x, input_y);

        // The maximum distance a key can have to be considered as closer is twice the cell height/width to avoid very far away buttons to get pressed
        let max_deltas = (2.0 * self.cell_radius, 2.0 * self.cell_radius);
        // Calculate the distance between the user interaction and each of the keys
        // If the distance exceeds the maximum delta in one of the dimensions, the distance is maximum so the button can never be closer
        for ((key_coordinate_x, key_coodinate_y), key) in &self.key_coordinates {
            let distance_new_point = View::get_distance(
                (*key_coordinate_x, *key_coodinate_y),
                (input_x, input_y),
                max_deltas,
            );
            // If the distance is closer than the current distance, the key is the new closest_key
            if distance_new_point < closest_distance {
                closest_key = Some(key);
                closest_distance = distance_new_point;
            }
        }

        println!("Closest key {:?}", closest_key);
        closest_key
    }

    /// Calculate the distance between point A and point B
    // If the distance exceeds the maximum delta in one of the dimensions, the distance is maximum
    fn get_distance(point_a: (f64, f64), point_b: (f64, f64), max_delta: (f64, f64)) -> f64 {
        let delta_x = (point_a.0 - point_b.0).abs();
        let delta_y = (point_a.1 - point_b.1).abs();
        if delta_x >= max_delta.0 || delta_y >= max_delta.1 {
            f64::MAX
        } else {
            let tmp = delta_x.powi(2) + delta_y.powi(2);
            tmp.sqrt()
        }
    }

    pub fn get_row_to_column_ratio(&self) -> f64 {
        self.row_to_column_ratio
    }
}
