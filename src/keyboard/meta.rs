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
pub struct KeyMeta {
    pub actions: HashMap<Interaction, Vec<KeyAction>>,
    pub key_display: KeyDisplay,
    pub outline: Outline,
    pub popup: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
}

impl KeyMeta {
    fn from(key_id: &str, key_deserialized: Option<&KeyDeserialized>) -> KeyMeta {
        let mut key_meta = KeyMeta::default(key_id);
        if let Some(key_deserialized) = key_deserialized {
            if let Some(deserialized_actions) = &key_deserialized.actions {
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

    fn default(key_id: &str) -> KeyMeta {
        let key_id = key_id.to_string();
        let mut actions = HashMap::new();
        actions.insert(
            Interaction::Tap(TapDuration::Short, TapMotion::Release),
            vec![KeyAction::EnterString(key_id.clone())],
        );
        let mut long_press_str = key_id.clone();
        if key_id.len() == 1 {
            long_press_str.make_ascii_uppercase();
        }
        actions.insert(
            Interaction::Tap(TapDuration::Long, TapMotion::Release),
            vec![KeyAction::EnterString(long_press_str)],
        );
        let key_display = KeyDisplay::Text(key_id);
        let outline = Outline::Standard;
        let popup = None;
        let styles = None;

        KeyMeta {
            actions,
            key_display,
            outline,
            popup,
            styles,
        }
    }

    fn make_interaction_keyaction_hashmap(
        deserialized_actions: &HashMap<KeyEvent, Vec<KeyAction>>,
    ) -> HashMap<Interaction, Vec<KeyAction>> {
        let mut actions = HashMap::new();
        for (key_event, key_action_vec) in deserialized_actions {
            // All actions are executed on release except for the toggle_keycode with a longpress
            // Then one action is created for the long_press and one for the long_press_release
            let mut press_action_vec = Vec::new();
            let mut release_action_vec = Vec::new();
            for action in key_action_vec {
                match action {
                    KeyAction::ToggleKeycode(_) => {
                        press_action_vec.push(action.clone());
                        release_action_vec.push(action.clone());
                    }
                    KeyAction::Modifier(_) => press_action_vec.push(action.clone()),

                    KeyAction::EnterKeycode(_)
                    | KeyAction::EnterString(_)
                    | KeyAction::SwitchView(_)
                    | KeyAction::TempSwitchView(_)
                    | KeyAction::SwitchLayout(_)
                    | KeyAction::TempSwitchLayout(_)
                    | KeyAction::Erase
                    | KeyAction::OpenPopup => release_action_vec.push(action.clone()),
                }
            }

            if !press_action_vec.is_empty() {
                let interaction_press = match key_event {
                    KeyEvent::ShortPress => Interaction::Tap(TapDuration::Short, TapMotion::Press),
                    KeyEvent::LongPress => Interaction::Tap(TapDuration::Long, TapMotion::Press),
                };
                actions.insert(interaction_press, press_action_vec);
            }

            if !release_action_vec.is_empty() {
                let interaction_release = match key_event {
                    KeyEvent::ShortPress => {
                        Interaction::Tap(TapDuration::Short, TapMotion::Release)
                    }
                    KeyEvent::LongPress => Interaction::Tap(TapDuration::Long, TapMotion::Release),
                };
                actions.insert(interaction_release, release_action_vec);
            }
        }
        actions.shrink_to_fit();
        actions
    }
}

#[derive(Debug)]
pub struct LayoutMeta {
    pub views: HashMap<String, KeyArrangement>,
    pub keys: HashMap<String, KeyMeta>,
}

impl LayoutMeta {
    pub fn new() -> HashMap<String, LayoutMeta> {
        let mut layout_meta = HashMap::new();
        let layout_deserialized = LayoutYamlParser::get_layouts();
        for (layout_name, layout_deserialized) in layout_deserialized {
            info!("LayoutMeta created for layout: {}", layout_name);
            layout_meta.insert(layout_name, LayoutMeta::from(layout_deserialized));
        }
        layout_meta
    }

    // KeyMeta for all needed keys is created and the string of keyids is converted to a hashmap with the location and size of each key
    fn from(layout_deserialized: LayoutDeserialized) -> LayoutMeta {
        let mut views = HashMap::new();
        let mut keys = HashMap::new();
        for (view_name, key_arrangement) in layout_deserialized.views {
            let keys_for_view =
                LayoutMeta::get_key_meta_for_all_keys(&key_arrangement, &layout_deserialized.keys);
            for (key_id, key_meta) in keys_for_view {
                keys.insert(key_id, key_meta); // Could use map1.extend(map2); instead
            }
            let view = KeyArrangement::from(&key_arrangement, &keys);
            views.insert(view_name, view);
        }
        LayoutMeta { views, keys }
    }

    fn get_key_meta_for_all_keys(
        key_arrangement_deserialized: &[KeyIds],
        key_meta: &HashMap<String, KeyDeserialized>,
    ) -> HashMap<String, KeyMeta> {
        let mut keys = HashMap::new();
        for row in key_arrangement_deserialized {
            for key_id in row.split_whitespace() {
                let key_meta = KeyMeta::from(key_id, key_meta.get(key_id));
                keys.insert(key_id.to_string(), key_meta);
            }
        }
        keys
    }
}

#[derive(Debug)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct KeyArrangement {
    pub key_arrangement: HashMap<String, Location>,
    pub no_rows: i32,
    pub no_columns: i32,
}
impl KeyArrangement {
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

    fn get_uncentered_key_arrangement(
        key_arrangement_deserialized: &[KeyIds],
        key_meta: &HashMap<String, KeyMeta>,
    ) -> (HashMap<String, Location>, Vec<i32>) {
        let mut key_arrangement = HashMap::new();
        let mut row_widths = Vec::new(); // Tracks width of the rows to later center the rows
        for (row_no, row) in key_arrangement_deserialized.iter().enumerate() {
            row_widths.insert(row_no, 0);
            for key_id in row.split_whitespace() {
                let (x, y) = (row_widths[row_no], row_no as i32);
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

    fn get_centered_key_arrangement(
        uncentered_key_arrangement: HashMap<String, Location>,
        row_widths: &[i32],
    ) -> (HashMap<String, Location>, i32, i32) {
        let no_columns = row_widths.iter().max().unwrap();
        let no_rows = row_widths.len() as i32;
        let mut key_arrangement_centered = HashMap::new();
        for (
            key,
            Location {
                x,
                y,
                width,
                height,
            },
        ) in uncentered_key_arrangement
        {
            let new_x = (no_columns - row_widths[y as usize]) / 2 + x;
            let new_location = Location {
                x: new_x,
                y,
                width,
                height,
            };
            key_arrangement_centered.insert(key, new_location);
        }
        (key_arrangement_centered, *no_columns, no_rows)
    }
}
