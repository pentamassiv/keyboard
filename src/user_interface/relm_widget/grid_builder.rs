use gtk::{
    ButtonExt, ContainerExt, Grid, GridExt, Popover, Stack, StackExt, StyleContextExt,
    ToggleButton, WidgetExt,
};
use std::collections::HashMap;

use crate::config::directories;
use crate::keyboard::{KeyArrangement, KeyDisplay, KeyMeta, LayoutMeta, Location};

/// Buttons are identified by a tuple of three strings '(layout_name, view_name, key_id)'
pub type ButtonId = (String, String, String);

pub struct GridBuilder;
impl GridBuilder {
    pub fn make_stack(
        relm: &relm::Relm<crate::user_interface::Win>,
        layout_meta_hashmap: HashMap<String, LayoutMeta>,
    ) -> (Stack, HashMap<ButtonId, (ToggleButton, Option<Popover>)>) {
        let stack = Stack::new();
        let mut hashmap_with_key_refs = HashMap::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        for (layout_name, layout_meta) in layout_meta_hashmap {
            for (view_name, view_arrangement) in layout_meta.views {
                let grid_name = GridBuilder::make_grid_name(&layout_name, &view_name);
                let (grid, key_refs) =
                    GridBuilder::make_grid(relm, &view_arrangement, &layout_meta.keys);
                grid.get_style_context()
                    .add_class(&format!("grid_{}", grid_name));
                stack.add_named(&grid, &grid_name);
                info!("Added view named: '{}'", grid_name);
                for (key_id, button_popup) in key_refs {
                    hashmap_with_key_refs.insert(
                        (layout_name.clone(), view_name.clone(), key_id),
                        button_popup,
                    );
                }
            }
        }
        hashmap_with_key_refs.shrink_to_fit();
        (stack, hashmap_with_key_refs)
    }

    fn make_grid(
        relm: &relm::Relm<crate::user_interface::Win>,
        view_arrangement: &KeyArrangement,
        view_keys: &HashMap<String, KeyMeta>,
    ) -> (Grid, HashMap<String, (ToggleButton, Option<Popover>)>) {
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        let mut hashmap_with_key_refs = HashMap::new();
        for (key_id, location) in &view_arrangement.key_arrangement {
            let key_meta = view_keys.get(key_id).unwrap();
            let button = GridBuilder::make_button(&key_id, key_meta);
            let popover = GridBuilder::attach_popover(relm, &button, key_meta);
            let Location {
                x,
                y,
                width,
                height,
            } = location;
            grid.attach(&button, *x, *y, *width, *height);
            hashmap_with_key_refs.insert(key_id.to_string(), (button.clone(), popover));
        }
        (grid, hashmap_with_key_refs)
    }
    fn make_button(key_id: &str, key_meta: &KeyMeta) -> ToggleButton {
        let button = ToggleButton::new();
        button.set_label(key_id);
        button.set_hexpand(true);
        button.get_style_context().add_class("key");
        button
            .get_style_context()
            .add_class(&format!("key_{}", key_id));

        if let Some(style_classes) = &key_meta.styles {
            for style_classes in style_classes {
                button.get_style_context().add_class(style_classes);
            }
        }
        match &key_meta.key_display {
            KeyDisplay::Text(label_text) => button.set_label(&label_text),
            KeyDisplay::Image(icon_name) => {
                if let Some(icon_dir_abs) =
                    directories::get_absolute_path(directories::ICON_DIR_REL)
                {
                    let mut icon_path = icon_dir_abs;
                    icon_path.push(&icon_name);
                    let image = gtk::Image::from_file(&icon_path);
                    button.set_image(Some(&image));
                }
                button.set_always_show_image(true);
                button.set_label("");
            }
        }
        button
    }

    fn attach_popover(
        relm: &relm::Relm<crate::user_interface::Win>,
        button: &ToggleButton,
        key_meta: &KeyMeta,
    ) -> Option<Popover> {
        let mut popover_option = None;
        if let Some(popup) = &key_meta.popup {
            let popover = Popover::new(Some(button));
            let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            for popup_string in popup {
                let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                for popup_id in popup_string.split_whitespace() {
                    let new_popup_button = gtk::Button::new();
                    new_popup_button
                        .get_style_context()
                        .add_class("popover_key");
                    new_popup_button.set_label(popup_id);
                    h_box.add(&new_popup_button);
                    let tmp_popover_ref = popover.clone();
                    new_popup_button.connect_clicked(move |_| tmp_popover_ref.hide());
                    relm::connect!(
                        relm,
                        new_popup_button,
                        connect_button_release_event(clicked_button, _),
                        return (
                            Some(crate::user_interface::Msg::SubmitText(
                                clicked_button.get_label().unwrap().to_string()
                            )),
                            gtk::Inhibit(false)
                        )
                    );
                }
                v_box.add(&h_box);
            }
            popover.add(&v_box);
            popover_option = Some(popover);
        }
        popover_option
    }

    pub fn make_grid_name(layout_name: &str, view_name: &str) -> String {
        let mut layout_view_name = String::from(layout_name);
        layout_view_name.push('_'); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }
}
