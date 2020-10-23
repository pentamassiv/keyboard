// Imports from other crates
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs::File;

// Imports from other modules
use super::deserializer::LayoutSource;
use crate::config::fallback_layout::{FALLBACK_LAYOUT, FALLBACK_LAYOUT_NAME};

/// The ids of keys are written in a string, separated by SPACES
pub type KeyIds = String;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
/// Enumeration to differentiate a short from a long press
/// There are no variants to make the definition of a layout simpler
/// The KeyEvent will need to be translated
pub enum KeyEvent {
    #[serde(rename = "short_press")]
    ShortPress,
    #[serde(rename = "long_press")]
    LongPress,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
/// The activation of a key can cause different action to take place
pub enum KeyAction {
    /// Give feedback about the button being pressed/released.
    /// 'true' signals a press of the button
    /// 'false' signals its release
    /// This is automatically added and should not be used in the layout description (in the yaml file)
    FeedbackPressRelease(bool),
    #[serde(rename = "enter_keycode")]
    #[serde(deserialize_with = "from_str")] // Look up the keycode to translate the string to a keycode
    /// Enter a keycode
    /// In the yaml file where the keyboard is described, the keycode needs to be defined as a string
    /// The keycode for the string is then looked up (eg. 'SPACE' -> 57)
    /// Invalid keycodes will cause the the deserialization to fail so the error is not only noticed when the key is pressed, but immediatly when the application is started
    EnterKeycode(u32),

    #[serde(rename = "toggle_keycode")]
    #[serde(deserialize_with = "from_str")]
    /// Same as EnterKeycode, but the result of the action depends on the current state of the key.
    /// If it is currently pressed, it sends a request to release it
    /// If it is currently released, it sends a request to press it
    ToggleKeycode(u32),

    #[serde(rename = "enter_string")]
    /// Submit the string
    EnterString(String),

    #[serde(rename = "modifier")]
    /// Submit the modifier
    Modifier(Modifier),

    #[serde(rename = "switch_view")]
    /// Switch the view
    SwitchView(String),

    #[serde(rename = "temporarily_switch_view")]
    /// Temporarily switch the view. After the next key is pressed, the view will switch back
    TempSwitchView(String),

    #[serde(rename = "switch_layout")]
    /// Switch the layout
    SwitchLayout(String),

    #[serde(rename = "temporarily_switch_layout")]
    /// Temporarily switch the layout. After the next key is pressed, the layout will switch back
    TempSwitchLayout(String),

    #[serde(rename = "erase")]
    /// Erase the last char (NOT grapheme!)
    Erase,

    #[serde(rename = "open_popup")]
    /// Open the keys popup
    /// The content of the popup is defined by a different struct
    OpenPopup,
}

/// Tries to look up the numeric value of a keycode. If it is not valid, return an error
fn from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let mut keycode_str: String = Deserialize::deserialize(deserializer)?;
    keycode_str = keycode_str.to_ascii_uppercase(); // Necessary because all keys in the HashMap are uppercase
    input_event_codes_hashmap::KEY
        .get::<str>(&keycode_str)
        .map_or(Err(Error::custom("Not a valid keycode")), |keycode| {
            Ok(*keycode)
        })
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
/// Keys can display a text or they can show an image
pub enum KeyDisplay {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image(String),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// These values reflect how many cells in a grid a button should take. Standard is two, to allow a row to have its buttons be half a button shifted to the right
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
/// The different modifiers that are available
pub enum Modifier {
    Shift,
    Lock,
    Control,
    #[serde(alias = "Mod1")]
    Alt,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
/// Not all values of a key need to be described in the yaml file
/// After the key is deserialized, information needs to be added to build an actual key and its visual representation (button)
pub struct KeyDeserialized {
    pub actions: Option<HashMap<KeyEvent, Vec<KeyAction>>>, // The actions a key causes when activated
    pub key_display: Option<KeyDisplay>,                    // What the button displays as its label
    pub outline: Option<Outline>,                           // The dimension of the key (width)
    pub popup: Option<Vec<String>>, // The content of a popover that can be opened
    pub styles: Option<Vec<String>>, // Style classes that can get attatched to the key to easily style it
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// The deserialized root element describing an entire keyboard
pub struct LayoutDeserialized {
    pub views: HashMap<String, Vec<KeyIds>>,
    #[serde(rename = "buttons")]
    // Purism calls it buttons, renamed so users need to change less when using a layout from squeekboard
    pub keys: HashMap<String, KeyDeserialized>,
}

impl LayoutDeserialized {
    /// Try to deserialize the layout that is described in the source
    /// If it does not contain a valid description and error is returned
    pub fn from(source: LayoutSource) -> Result<(String, LayoutDeserialized), serde_yaml::Error> {
        let mut layout_name: String = String::from(FALLBACK_LAYOUT_NAME);
        let layout = match source {
            // If the source is a file path,..
            LayoutSource::YamlFile(path) => {
                // Use the file name as the name for the layout
                layout_name = String::from(path.file_stem().unwrap().to_str().unwrap());
                // The name 'previous' can not be used because it is used to signal that the layout is supposed to switch to the previous layout
                if layout_name == "previous" {
                    return Err(Error::custom("The layout can not be named 'previous'. That name is used internally and can not be used because it would never be possible to switch to this layout. Please chose a different name"));
                }
                // Try to open the file
                let file_descriptor: String = format!("{}", &path.display());
                let yaml_file = File::open(&file_descriptor).expect("No file found!");
                // and deserialize the layout
                serde_yaml::from_reader(yaml_file)
            }
            // If the source is the fallback string, try to deserialize the layout from the string
            LayoutSource::FallbackStr => serde_yaml::from_str(&FALLBACK_LAYOUT),
        };

        // If the deserialization was successful, return the layout and its name
        match layout {
            Ok(layout) => {
                info!("Successfully deserialized layout: {}", layout_name);
                Ok((layout_name, layout))
            }
            Err(err) => {
                info!(
                    "Error deserializing layout {}. Error description: {}",
                    layout_name, err
                );
                Err(err)
            }
        }
    }
}
