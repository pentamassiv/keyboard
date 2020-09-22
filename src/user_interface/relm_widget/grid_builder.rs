use crate::keyboard::{KeyArrangement, KeyDisplay, KeyMeta, LayoutMeta, Location};
use gtk::{Button, ButtonExt, Grid, GridExt, Stack, StackExt, StyleContextExt, WidgetExt};
use std::collections::HashMap;

pub const ICON_FOLDER: &str = "./data/icons/";

pub struct GridBuilder;
impl GridBuilder {
    pub fn make_stack(
        layout_meta_hashmap: HashMap<String, LayoutMeta>,
    ) -> (Stack, HashMap<String, Button>) {
        let stack = Stack::new();
        let mut hashmap_with_key_refs = HashMap::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        for (layout_name, layout_meta) in layout_meta_hashmap {
            for (view_name, view_arrangement) in layout_meta.views {
                let grid_name = GridBuilder::make_grid_name(&layout_name, &view_name);
                let (grid, key_refs) = GridBuilder::make_grid(&view_arrangement, &layout_meta.keys);
                stack.add_named(&grid, &grid_name);
                hashmap_with_key_refs.extend(key_refs)
            }
        }
        (stack, hashmap_with_key_refs)
    }

    /*
    pub struct Location {
        pub coordinate: (u32, u32),
        pub size: (u32, u32),
    }
    */

    fn make_grid(
        view_arrangement: &KeyArrangement,
        view_keys: &HashMap<String, KeyMeta>,
    ) -> (Grid, HashMap<String, Button>) {
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        let hashmap_with_key_refs = HashMap::new();
        for (key_id, location) in &view_arrangement.key_arrangement {
            let button = GridBuilder::make_button(&key_id, view_keys.get(key_id).unwrap());
            let Location {
                coordinate: (x, y),
                size: (width, height),
            } = location;
            grid.attach(&button, *x, *y, *width, *height);
        }
        (grid, hashmap_with_key_refs)
    }
    fn make_button(key_id: &str, key_meta: &KeyMeta) -> Button {
        let button = Button::new();
        button.set_label(key_id);
        button.set_hexpand(true);
        button.get_style_context().add_class("key");

        if let Some(style_classes) = &key_meta.styles {
            for style_classes in style_classes {
                button.get_style_context().add_class(style_classes);
            }
        }
        match &key_meta.key_display {
            KeyDisplay::Text(label_text) => button.set_label(&label_text),
            KeyDisplay::Image(icon_name) => {
                let mut icon_path = String::from(ICON_FOLDER);
                icon_path.push_str(&icon_name);
                let image = gtk::Image::from_file(&icon_path);
                button.set_image(Some(&image));
                button.set_always_show_image(true);
                button.set_label("");
            }
        }
        /*

                let popover = gtk::Popover::new(Some(&button));
        if let Some(popup) = &key_meta.popup {
                        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        for popup_string in popup {
                            let new_popup_button = gtk::Button::new();
                            new_popup_button.set_label(popup_string);
                            hbox.add(&new_popup_button);
                            let tmp_popover_ref = popover.clone();
                            new_popup_button.connect_clicked(move |_| tmp_popover_ref.hide());
                            /*relm::connect!(
                                relm,
                                new_popup_button,
                                connect_button_release_event(clicked_button, _),
                                return (
                                    Some(crate::user_interface::Msg::EnterString(
                                        clicked_button.get_label().unwrap().to_string(),
                                        false,
                                    )),
                                    gtk::Inhibit(false)
                                )
                            );*/
                        }
                        popover.add(&hbox);
                    }
        */

        button
    }

    pub fn make_grid_name(layout_name: &str, view_name: &str) -> String {
        let mut layout_view_name = String::from(layout_name);
        layout_view_name.push('_'); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }
}
