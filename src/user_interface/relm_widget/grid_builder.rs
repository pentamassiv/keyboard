// Imports from other crates
use gtk::{
    ButtonExt, ContainerExt, Grid, GridExt, Popover, Stack, StackExt, StyleContextExt,
    ToggleButton, WidgetExt,
};
use std::collections::HashMap;

// Imports from other modules
use crate::config::directories;
use crate::keyboard::{KeyArrangement, KeyDisplay, KeyMeta, LayoutMeta, Location};

/// Buttons are identified by a tuple of three strings '(layout_name, view_name, key_id)'
pub type ButtonId = (String, String, String);

pub struct GridBuilder;
impl GridBuilder {
    // Makes the 'gtk::Stack' that allows stores the grids of buttons for the layouts and allows switching between them
    // from the provided layout meta information. It also returns a HashMap to get the references to each button and its popover from the ButtonsId
    // This is necessary to change the buttons properties later on (eg set it's state to active/inactive) and to open it's popover
    pub fn make_stack(
        relm: &relm::Relm<crate::user_interface::Win>,
        layout_meta_hashmap: HashMap<String, LayoutMeta>,
    ) -> (Stack, HashMap<ButtonId, (ToggleButton, Option<Popover>)>) {
        // Make a new stack and a new HashMap
        let stack = Stack::new();
        let mut hashmap_with_key_refs = HashMap::new();
        stack.set_transition_type(gtk::StackTransitionType::None);
        // For each layout
        for (layout_name, layout_meta) in layout_meta_hashmap {
            // .. and each of its views
            for (view_name, view_arrangement) in layout_meta.views {
                // Make the name of the gid the layout consists of (necessary because the stack only allows setting the visible child by a string and not a tuple of
                // the layout name and the view)
                let grid_name = GridBuilder::make_grid_name(&layout_name, &view_name);
                // Make the grid for the layout from the arrangement of the keys and the meta info of the keys
                let (grid, key_refs) =
                    GridBuilder::make_grid(relm, &view_arrangement, &layout_meta.keys);
                grid.get_style_context()
                    .add_class(&format!("grid_{}", grid_name));
                // Add the grid to the stack
                stack.add_named(&grid, &grid_name);
                info!("Added view named: '{}'", grid_name);
                // Add all of the keys of the layout to the HashMap with the references to them
                for (key_id, button_popup) in key_refs {
                    hashmap_with_key_refs.insert(
                        (layout_name.clone(), view_name.clone(), key_id),
                        button_popup,
                    );
                }
            }
        }
        // Shrink the HashMap to its smalles size to reduce the memory usage
        // The HashMap will not changed after this but will not be freed until the program is ended
        hashmap_with_key_refs.shrink_to_fit();
        (stack, hashmap_with_key_refs)
    }

    /// Make the grid of buttons for a layout
    // It also returns a HashMap to get the references to each button and its popover from the ButtonsId
    // This is necessary to change the buttons properties later on (eg set it's state to active/inactive) and to open it's popover
    fn make_grid(
        relm: &relm::Relm<crate::user_interface::Win>,
        view_arrangement: &KeyArrangement,
        view_keys: &HashMap<String, KeyMeta>,
    ) -> (Grid, HashMap<String, (ToggleButton, Option<Popover>)>) {
        // Make a new grid and a new HashMap
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        let mut hashmap_with_key_refs = HashMap::new();
        // Make the keys
        for (key_id, location) in view_arrangement.get_key_arrangement() {
            // Get the key meta infos for the key id
            let key_meta = view_keys.get(key_id).unwrap();
            // Make a button from the key meta infos
            let button = GridBuilder::make_button(&key_id, key_meta);
            // Make a popover for the key
            let popover = GridBuilder::attach_popover(relm, &button, key_meta);
            // Add the key to the grid
            let Location {
                x,
                y,
                width,
                height,
            } = location;
            grid.attach(&button, *x, *y, *width, *height);
            // Insert the references to the HashMap
            hashmap_with_key_refs.insert(key_id.to_string(), (button.clone(), popover));
        }
        (grid, hashmap_with_key_refs)
    }

    /// Make a ToggleButton from the key meta infos
    fn make_button(key_id: &str, key_meta: &KeyMeta) -> ToggleButton {
        // Make a new ToggleButton
        let button = ToggleButton::new();
        button.set_hexpand(true);
        button.get_style_context().add_class("key");
        button
            .get_style_context()
            .add_class(&format!("key_{}", key_id));

        // Add style classes, if any were specified
        if let Some(style_classes) = &key_meta.styles {
            for style_classes in style_classes {
                button.get_style_context().add_class(style_classes);
            }
        }
        // Set the button to display a label or an icon
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
                    button.set_always_show_image(true);
                } else {
                    error!(
                        "Unable to locate the image to display for button '{}'",
                        key_id
                    );
                }
            }
        }
        button
    }

    /// Attaches a popover to the toggle button, if there is one defined in the key meta info. If no popover is supposed to get added, it returns 'None
    fn attach_popover(
        relm: &relm::Relm<crate::user_interface::Win>,
        button: &ToggleButton,
        key_meta: &KeyMeta,
    ) -> Option<Popover> {
        let mut popover_option = None;
        // If a popover is specified..
        if let Some(popup) = &key_meta.popup {
            // Make a new popover
            let popover = Popover::new(Some(button));
            // Add a vertical box so the popover can contain multiple rows
            let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            for popup_string in popup {
                // Add a horizontal box so the popover can contain multiple columns
                let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                // Separate the popup string by the whitespace and for each of the popup ids..
                for popup_id in popup_string.split_whitespace() {
                    // Make a new button
                    let new_popup_button = gtk::Button::new();
                    new_popup_button
                        .get_style_context()
                        .add_class("popover_key");
                    // Set its label to its id
                    new_popup_button.set_label(popup_id);
                    // Add the button to the horizontal box
                    h_box.add(&new_popup_button);
                    let tmp_popover_ref = popover.clone();
                    // Request the UI to submit the label of the button if it gets clicked
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
                // Add the horizontal box to the vertical box
                v_box.add(&h_box);
            }
            popover.add(&v_box);
            popover_option = Some(popover);
        }
        popover_option
    }

    /// Make the name of the grid.
    /// The returned name is the layout and the view name but separated by an underscore
    /// (necessary because the stack only allows setting the visible child by a string and not a tuple of
    /// the layout name and the view)
    pub fn make_grid_name(layout_name: &str, view_name: &str) -> String {
        let mut layout_view_name = String::from(layout_name);
        layout_view_name.push('_'); //Separator Character
        layout_view_name.push_str(view_name);
        layout_view_name
    }
}
