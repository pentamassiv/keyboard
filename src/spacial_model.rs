use std::collections::HashMap;

#[derive(Debug)]
pub struct SpacialModel {
    pub spacial_views: HashMap<(String, String), SpacialModelView>,
}
impl SpacialModel {
    pub fn new() -> SpacialModel {
        SpacialModel {
            spacial_views: HashMap::new(),
        }
    }
    pub fn add_spacial_model(
        &mut self,
        layout_name: &str,
        view_name: &str,
        spacial_model_view: SpacialModelView,
    ) {
        self.spacial_views.insert(
            (String::from(layout_name), String::from(view_name)),
            spacial_model_view,
        );
    }
    pub fn get_closest_button(
        &self,
        layout_name: &str,
        view_name: &str,
        x: i32,
        y: i32,
    ) -> Option<gtk::Button> {
        if let Some(spacial_model_view) = self
            .spacial_views
            .get(&(layout_name.to_string(), view_name.to_string()))
        {
            spacial_model_view.get_closest_button(x, y)
        } else {
            None
        }
    }
}
#[derive(Debug)]
pub struct SpacialModelView {
    button_coordinates: HashMap<(i32, i32), gtk::Button>,
}
impl SpacialModelView {
    pub fn add_button_coordinate(&mut self, x: i32, y: i32, button: gtk::Button) {
        self.button_coordinates.insert((x, y), button);
    }

    fn get_closest_button(&self, input_x: i32, input_y: i32) -> Option<gtk::Button> {
        let mut closest_button = None;
        let mut closest_distance = i32::MAX;
        for (x, y) in self.button_coordinates.keys() {
            let distance_new_point = self.get_distance((*x, *y), (input_x, input_y));
            if distance_new_point < closest_distance {
                closest_button = self.button_coordinates.get(&(*x, *y));
                closest_distance = distance_new_point;
            }
        }
        let mut result = None;
        if let Some(button) = closest_button {
            let buttons = button.clone();
            result = Some(buttons);
        }
        result
    }

    fn get_distance(&self, coordinate_a: (i32, i32), coordinate_b: (i32, i32)) -> i32 {
        let delta_x = (coordinate_a.0 - coordinate_b.0).abs();
        let delta_y = (coordinate_a.1 - coordinate_b.1).abs();
        let tmp = (delta_x.pow(2) + delta_y.pow(2)) as f64;
        tmp.sqrt() as i32
    }

    pub fn new() -> SpacialModelView {
        SpacialModelView {
            button_coordinates: HashMap::new(),
        }
    }
}
