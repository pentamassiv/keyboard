use super::deserializer::LayoutSource;
use crate::config::fallback_layout::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;

/// Keys are embedded in a single string
pub type KeyIds = String;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum KeyEvent {
    #[serde(rename = "short_press")]
    ShortPress,
    #[serde(rename = "long_press")]
    LongPress,
    #[serde(rename = "swipe")] // TODO: Should not be needed, delete?
    Swipe, // TODO: Should not be needed, delete?
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum KeyAction {
    #[serde(rename = "enter_keycode")]
    EnterKeycode(String),
    #[serde(rename = "toggle_keycode")]
    ToggleKeycode(String),
    #[serde(rename = "enter_string")]
    EnterString(String),
    #[serde(rename = "modifier")]
    Modifier(Modifier),
    #[serde(rename = "switch_view")]
    SwitchView(String),
    #[serde(rename = "temporarily_switch_view")]
    TempSwitchView(String),
    #[serde(rename = "switch_layout")]
    SwitchLayout(String),
    #[serde(rename = "temporarily_switch_layout")]
    TempSwitchLayout(String),
    #[serde(rename = "erase")]
    Erase,
    #[serde(rename = "open_popup")]
    OpenPopup,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum KeyDisplay {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image(String),
}

// These values reflect how many spaces in the grid of buttons the outline should take. That's why it needs to be an integer value
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Outline {
    #[serde(rename = "standard")]
    Standard = 2,
    #[serde(rename = "half")]
    Half = 1,
    #[serde(rename = "one_and_a_half")]
    OneAndAHalf = 3,
    #[serde(rename = "double")]
    Double = 4,
    #[serde(rename = "quadruple")]
    Quadruple = 8,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub enum Modifier {
    Control,
    Shift,
    Lock,
    #[serde(alias = "Mod1")]
    Alt,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct KeyDeserialized {
    pub actions: Option<HashMap<KeyEvent, Vec<KeyAction>>>,
    pub key_display: Option<KeyDisplay>,
    pub outline: Option<Outline>,
    pub popup: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
}

/// The root element describing an entire keyboard
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LayoutDeserialized {
    pub views: HashMap<String, Vec<KeyIds>>,
    #[serde(rename = "buttons")]
    // Purism calls it buttons, renamed so users need to change less when using a layout from squeekboard
    pub keys: HashMap<String, KeyDeserialized>,
}

impl LayoutDeserialized {
    pub fn from(source: LayoutSource) -> Result<(String, LayoutDeserialized), serde_yaml::Error> {
        let mut layout_name: String = String::from(FALLBACK_LAYOUT_NAME);
        let layout = match source {
            LayoutSource::YamlFile(path) => {
                layout_name = String::from(path.file_stem().unwrap().to_str().unwrap());
                let file_descriptor: String = format!("{}", &path.display());
                let yaml_file = File::open(&file_descriptor).expect("No file found!");
                serde_yaml::from_reader(yaml_file)
            }
            LayoutSource::FallbackStr => serde_yaml::from_str(&FALLBACK_LAYOUT),
        };

        match layout {
            Ok(layout) => Ok((layout_name, layout)),
            Err(err) => Err(err),
        }
    }
}
