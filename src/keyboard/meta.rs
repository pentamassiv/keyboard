// Imports from other crates
use std::collections::HashMap;

// Imports from other modules
use crate::keyboard::{Interaction, TapDuration, TapMotion};

// Modules
mod deserialized_structs;
mod deserializer;
use deserialized_structs::{KeyDeserialized, KeyIds, LayoutDeserialized};
use deserializer::LayoutYamlParser;

// Re-exports
pub use deserialized_structs::{KeyAction, KeyDisplay, KeyEvent, Modifier, Outline};

#[derive(Debug)]
/// Struct to save all information needed to build a key and its representation as a button
/// In comparison to the KeyDeserialized struct, not specified information is set tof default values
pub struct KeyMeta {
    pub actions: HashMap<Interaction, Vec<KeyAction>>,
    pub key_display: KeyDisplay,
    pub outline: Outline,
    pub popup: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
}

impl KeyMeta {
    /// Convert the KeyDeserialized struct to a KeyMeta struct. This is done by copying the information that was deserialized and supplementing it with default values
    fn from(key_id: &str, key_deserialized: Option<&KeyDeserialized>) -> KeyMeta {
        // Make a default KeyMeta struct
        let mut key_meta = KeyMeta::default(key_id);
        // If some of the information about the key was provided in the layout definition file, overwrite the default values
        if let Some(key_deserialized) = key_deserialized {
            if let Some(deserialized_actions) = &key_deserialized.actions {
                // The HashMap<KeyEvent, Vec<KeyAction>> gets transformed to HashMap<Interaction, Vec<KeyAction>>
                key_meta.actions =
                    KeyMeta::make_interaction_keyaction_hashmap(deserialized_actions);
            };
            if let Some(deserialized_key_display) = &key_deserialized.key_display {
                key_meta.key_display = deserialized_key_display.clone();
            };
            if let Some(deserialized_outline) = &key_deserialized.outline {
                key_meta.outline = *deserialized_outline;
            };
            if key_deserialized.popup.is_some() {
                key_meta.popup = key_deserialized.popup.clone();
            };
            if key_deserialized.styles.is_some() {
                key_meta.styles = key_deserialized.styles.clone();
            };
        }
        key_meta
    }

    // Returns a default KeyMeta struct
    fn default(key_id: &str) -> KeyMeta {
        let key_id = key_id.to_string();
        let mut actions = HashMap::new();
        // If the key is pressed (short), its key_id is submitted
        actions.insert(
            Interaction::Tap(TapDuration::Short, TapMotion::Release),
            vec![KeyAction::EnterString(key_id.clone())],
        );
        let mut long_press_str = key_id.clone();
        if key_id.len() == 1 {
            long_press_str.make_ascii_uppercase();
        }
        // If it was pressed for a longer time, its capitalized key_id is sent
        actions.insert(
            Interaction::Tap(TapDuration::Long, TapMotion::Release),
            vec![KeyAction::EnterString(long_press_str)],
        );
        // The default to display the key is with its label set to its id
        let key_display = KeyDisplay::Text(key_id);
        // The outline is Standard
        let outline = Outline::Standard;
        // No popover is added
        let popup = None;
        // No css style classes are added
        let styles = None;

        KeyMeta {
            actions,
            key_display,
            outline,
            popup,
            styles,
        }
    }

    /// Transforms the HashMap<KeyEvent, Vec<KeyAction>> in HashMap<Interaction, Vec<KeyAction>>
    /// This is done because for the user it is easer to define KeyEvents that cause a KeyAction, but the Keyboard struct uses Interactions instead
    /// Some actions are executed when the button is pressed and others get executed when they are released. This is also taken care of in this method
    fn make_interaction_keyaction_hashmap(
        deserialized_actions: &HashMap<KeyEvent, Vec<KeyAction>>,
    ) -> HashMap<Interaction, Vec<KeyAction>> {
        let mut actions = HashMap::new();
        // Transform all entries...
        for (key_event, key_action_vec) in deserialized_actions {
            // All actions are executed on release
            // Except for the toggle_keycode with a long press, then one action is created for the long press and one for its release
            // And except for the Modifier action. This action gets activated with the press
            // Get the duration of the tap
            let tap_duration = match key_event {
                KeyEvent::ShortPress => TapDuration::Short,
                KeyEvent::LongPress => TapDuration::Long,
            };
            for action in key_action_vec {
                let mut activate_when_pressed = false;
                let mut activate_when_released = false;
                match action {
                    // There should never be a FeedbackPressRelease KeyAction defined by the user. If it is, it will be ignored
                    KeyAction::FeedbackPressRelease(_) => {}
                    // The ToggleKeycode causes a toggle when the button is pressed and when it is released
                    KeyAction::ToggleKeycode(_) => {
                        activate_when_pressed = true;
                        activate_when_released = true;
                    }
                    // The Modifier key action is executed when the key is pressed
                    KeyAction::Modifier(_) => {
                        activate_when_pressed = true;
                    }

                    // All other key actions are executed when the key is released
                    KeyAction::EnterKeycode(_)
                    | KeyAction::EnterString(_)
                    | KeyAction::SwitchView(_)
                    | KeyAction::TempSwitchView(_)
                    | KeyAction::SwitchLayout(_)
                    | KeyAction::TempSwitchLayout(_)
                    | KeyAction::Erase
                    | KeyAction::OpenPopup => {
                        activate_when_released = true;
                    }
                }
                // If the action gets executed when it is pressed, add it to the actions
                if activate_when_pressed {
                    Self::add_tap_action_to_hashmap(
                        tap_duration,
                        TapMotion::Press,
                        action.clone(),
                        &mut actions,
                    );
                }
                // If the action gets executed when it is released, add it to the actions
                if activate_when_released {
                    Self::add_tap_action_to_hashmap(
                        tap_duration,
                        TapMotion::Release,
                        action.clone(),
                        &mut actions,
                    );
                }
            }
        }
        // Shrink the HashMap to save memory
        actions.shrink_to_fit();
        actions
    }

    pub fn add_tap_action_to_hashmap(
        duration: TapDuration,
        motion: TapMotion,
        action: KeyAction,
        hashmap: &mut HashMap<Interaction, Vec<KeyAction>>,
    ) {
        let interaction = Interaction::Tap(duration, motion);
        let actions_vec = hashmap.get_mut(&interaction);
        if let Some(actions_vec) = actions_vec {
            actions_vec.push(action);
        } else {
            let actions_vec = vec![action];
            hashmap.insert(interaction, actions_vec);
        }
    }
}

#[derive(Debug)]
/// This struct contains the arrangement of all its keys and all the information to build them
pub struct LayoutMeta {
    pub views: HashMap<String, KeyArrangement>,
    pub keys: HashMap<String, KeyMeta>,
}

impl LayoutMeta {
    /// Search for layout definitions in a path, deserialize them, complete the information and return a HashMap of eachs layouts name with its LayoutMeta
    pub fn deserialize() -> HashMap<String, LayoutMeta> {
        let mut layout_meta = HashMap::new();
        // Deserialize all available layouts from a file path
        let layout_deserialized = LayoutYamlParser::get_layouts();
        for (layout_name, layout_deserialized) in layout_deserialized {
            info!("LayoutMeta created for layout: {}", layout_name);
            layout_meta.insert(layout_name, LayoutMeta::from(layout_deserialized));
        }
        layout_meta
    }

    /// Transforms a LayoutDeserialized to a LayoutMeta. This is done by creating the KeyMeta for all needed keys.
    /// Also the string of key_ids is converted to a hashmap with the location and size of each key
    fn from(layout_deserialized: LayoutDeserialized) -> LayoutMeta {
        let mut views = HashMap::new();
        let mut keys = HashMap::new();
        // For each view..
        for (view_name, key_arrangement) in layout_deserialized.views {
            // The KeyMeta for all its keys is created and added to the HashMap
            let keys_for_view =
                LayoutMeta::get_key_meta_for_all_keys(&key_arrangement, &layout_deserialized.keys);
            keys.extend(keys_for_view);
            //for (key_id, key_meta) in keys_for_view {
            //keys.insert(key_id, key_meta); // Could use map1.extend(map2); instead
            //}
            let view = KeyArrangement::from(&key_arrangement, &keys);
            views.insert(view_name, view);
        }
        LayoutMeta { views, keys }
    }

    /// Gets the KeyMeta for all keys
    /// If a KeyDeserialized of a key exists, it is used to build the KeyMeta, if not a default KeyMeta is created
    fn get_key_meta_for_all_keys(
        key_arrangement_deserialized: &[KeyIds],
        key_meta: &HashMap<String, KeyDeserialized>,
    ) -> HashMap<String, KeyMeta> {
        let mut keys = HashMap::new();
        // For all keys..
        for row in key_arrangement_deserialized {
            for key_id in row.split_whitespace() {
                // create the KeyMeta for that key_id and add it to the HashMap
                let key_meta = KeyMeta::from(key_id, key_meta.get(key_id));
                keys.insert(key_id.to_string(), key_meta);
            }
        }
        keys
    }
}

#[derive(Debug, Copy, Clone)]
/// Location of a key
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
/// Stores how the keys are arranged
pub struct KeyArrangement {
    key_arrangement: HashMap<String, Location>,
    no_rows: i32,
    no_columns: i32,
}

impl KeyArrangement {
    /// Create a new KeyArrangement from the strings of KeyIds and the information about the keys
    pub fn from(
        key_arrangement_deserialized: &[KeyIds],
        key_meta: &HashMap<String, KeyMeta>,
    ) -> KeyArrangement {
        let (uncentered_key_arrangement, row_widths) =
            KeyArrangement::get_uncentered_key_arrangement(key_arrangement_deserialized, key_meta);
        let (centered_key_arrangement, no_columns, no_rows) =
            KeyArrangement::get_centered_key_arrangement(uncentered_key_arrangement, &row_widths);
        KeyArrangement {
            key_arrangement: centered_key_arrangement,
            no_rows,
            no_columns,
        }
    }

    // Return the number of rows
    pub fn get_no_rows(&self) -> i32 {
        self.no_rows
    }

    // Return the number of columns
    pub fn get_no_columns(&self) -> i32 {
        self.no_columns
    }

    // Return a reference of the KeyArrangement
    pub fn get_key_arrangement(&self) -> &HashMap<String, Location> {
        &self.key_arrangement
    }

    // Create the key arrangement, this needs to get centered afterwards because not all rows have to contain the same amount of keys
    // The layout definition in files are simpler this way because no blank keys or something alike need to be described to achieve centered rows
    fn get_uncentered_key_arrangement(
        key_arrangement_deserialized: &[KeyIds],
        key_meta: &HashMap<String, KeyMeta>,
    ) -> (HashMap<String, Location>, Vec<i32>) {
        let mut key_arrangement = HashMap::new();
        let mut row_widths = Vec::new(); // Tracks width of the rows to later center the rows
        for (row_no, row) in key_arrangement_deserialized.iter().enumerate() {
            // For each of the rows of strings of key ids
            row_widths.insert(row_no, 0);
            // Get the individual key ids and for each of them..
            for key_id in row.split_whitespace() {
                // Calculate where in the grid the button should be placed
                let (x, y) = (row_widths[row_no], row_no as i32);

                // Calculate how many cells the button should be wide/high
                let (width, height) = (
                    key_meta
                        .get(key_id)
                        .expect("KeyMeta should have been completed")
                        .outline as i32,
                    1,
                );
                let location = Location {
                    x,
                    y,
                    width,
                    height,
                };
                row_widths[row_no] += width;
                key_arrangement.insert(key_id.to_string(), location);
            }
        }
        (key_arrangement, row_widths)
    }

    /// If a row is not centered, it moves each of its keys along to the right, to get it centered
    fn get_centered_key_arrangement(
        mut uncentered_key_arrangement: HashMap<String, Location>,
        row_widths: &[i32],
    ) -> (HashMap<String, Location>, i32, i32) {
        let no_columns = row_widths.iter().max().unwrap();
        let no_rows = row_widths.len() as i32;

        // Moves the x coordinate to center the arrangement
        for (
            _,
            Location {
                x,
                y,
                width: _,
                height: _,
            },
        ) in uncentered_key_arrangement.iter_mut()
        {
            *x += (no_columns - row_widths[*y as usize]) / 2;
        }
        (uncentered_key_arrangement, *no_columns, no_rows)
    }
}
