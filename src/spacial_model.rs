use std::collections::HashMap;

pub struct SpacialModel {
    pub grid_dimenstions: HashMap<String, (usize, usize)>,
}
impl SpacialModel {
    pub fn new() -> SpacialModel {
        SpacialModel {
            grid_dimenstions: HashMap::new(),
        }
    }
    pub fn add_spacial_model(
        &mut self,
        layout_name: String,
        view_name: String,
        dimensions: (usize, usize),
    ) {
        self.grid_dimenstions
            .insert(crate::user_interface::Keyboard::make_layout_view_name(&layout_name, &view_name), dimensions);
    }
}

