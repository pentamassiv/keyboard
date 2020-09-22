use crate::keyboard::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct View {
    key_coordinates: HashMap<(i32, i32), Key>,
}

impl View {
    pub fn from(key_arrangement: &KeyArrangement, key_meta: &HashMap<String, KeyMeta>) -> View {
        let mut key_coordinates = HashMap::new();
        // Get the name and location and size of each key that will be in this view
        for (key_name, location) in &key_arrangement.key_arrangement {
            // Make a new key based on the key meta information
            let key = Key::from(&key_name, key_meta.get(key_name).unwrap());
            // The keys will be arranged in a grid so if a key has a size of e.g. two,
            // a clone of the key needs to be placed in each of the two cells that the wide key would cover
            for width in 0..location.size.0 {
                // The same is true for the height of the key
                for height in 0..location.size.1 {
                    let (x, y) = location.coordinate;
                    key_coordinates.insert((x + width, y + height), key.clone());
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
            let distance_new_point = self.get_distance((*x, *y), (input_x, input_y));
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

    fn get_distance(&self, coordinate_a: (i32, i32), coordinate_b: (i32, i32)) -> i32 {
        let delta_x = (coordinate_a.0 - coordinate_b.0).abs();
        let delta_y = (coordinate_a.1 - coordinate_b.1).abs();
        let tmp = (delta_x.pow(2) + delta_y.pow(2)) as f64;
        tmp.sqrt() as i32
    }
}
