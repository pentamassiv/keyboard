use crate::keyboard;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use tempfile::tempfile;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::Main;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

#[derive(Debug, PartialEq, Clone)]
pub enum SubmitError {
    /// Virtual keyboard proxy was dropped and is no longer alive
    NotAlive,
    /// The keycode was invalid
    InvalidKeycode,
}

#[derive(Debug, PartialEq, Clone)]
pub enum KeyMotion {
    Press = 1,
    Release = 0,
}

bitflags! {
    /// Maps the names of the modifiers to their bitcodes (From squeekboard)
    /// From https://www.x.org/releases/current/doc/kbproto/xkbproto.html#Keyboard_State
    pub struct ModifiersBitflag: u32 {
        const NO_MODIFIERS = 0x0;
        const SHIFT = 0x1;
        const LOCK = 0x2;
        const CONTROL = 0x4;
        /// Alt
        const MOD1 = 0x8;
        const MOD2 = 0x10;
        const MOD3 = 0x20;
        /// Meta
        const MOD4 = 0x40;
        /// AltGr
        const MOD5 = 0x80;
    }
}

impl From<keyboard::Modifier> for ModifiersBitflag {
    fn from(item: keyboard::Modifier) -> Self {
        match item {
            keyboard::Modifier::Shift => ModifiersBitflag::SHIFT,
            keyboard::Modifier::Lock => ModifiersBitflag::LOCK,
            keyboard::Modifier::Control => ModifiersBitflag::CONTROL,
            keyboard::Modifier::Alt => ModifiersBitflag::MOD1,
            keyboard::Modifier::Mod2 => ModifiersBitflag::MOD2,
            keyboard::Modifier::Mod3 => ModifiersBitflag::MOD3,
            keyboard::Modifier::Mod4 => ModifiersBitflag::MOD4,
            keyboard::Modifier::Mod5 => ModifiersBitflag::MOD5,
        }
    }
}

pub struct VKService {
    base_time: std::time::Instant,
    pressed_keys: HashSet<u32>,
    pressed_modifiers: ModifiersBitflag,
    virtual_keyboard: Main<ZwpVirtualKeyboardV1>,
}

impl VKService {
    pub fn new(seat: &WlSeat, vk_mgr: Main<ZwpVirtualKeyboardManagerV1>) -> VKService {
        let base_time = Instant::now();
        let pressed_keys = HashSet::new();
        let pressed_modifiers = ModifiersBitflag::NO_MODIFIERS;
        let virtual_keyboard = vk_mgr.create_virtual_keyboard(&seat);

        let vk_service = VKService {
            base_time,
            pressed_keys,
            pressed_modifiers,
            virtual_keyboard,
        };
        info!("VKService created");
        vk_service.init_virtual_keyboard();
        vk_service
    }

    fn init_virtual_keyboard(&self) {
        let src = super::keymap::KEYMAP;
        let keymap_size = super::keymap::KEYMAP.len();
        let keymap_size_u32: u32 = keymap_size.try_into().unwrap(); // Convert it from usize to u32, panics if it is not possible
        let keymap_size_u64: u64 = keymap_size.try_into().unwrap(); // Convert it from usize to u64, panics if it is not possible
        let mut keymap_file = tempfile().expect("Unable to create tempfile");
        // Allocate space in the file first
        keymap_file.seek(SeekFrom::Start(keymap_size_u64)).unwrap();
        keymap_file.write_all(&[0]).unwrap();
        keymap_file.seek(SeekFrom::Start(0)).unwrap();
        let mut data = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&keymap_file)
                .expect("Could not access data from memory mapped file")
        };
        data[..src.len()].copy_from_slice(src.as_bytes());
        let keymap_raw_fd = keymap_file.into_raw_fd();
        self.virtual_keyboard
            .keymap(1, keymap_raw_fd, keymap_size_u32);
        info!("VKService initialized the keyboard");
    }

    fn get_time(&self) -> u32 {
        let duration = self.base_time.elapsed();
        let time = duration.as_millis();
        time.try_into().unwrap()
    }

    pub fn release_all_keys_and_modifiers(&mut self) -> Result<(), SubmitError> {
        let result_key_release = self.release_all_keys();
        let result_modifiert_release = self.release_all_modifiers();
        if result_key_release.is_ok() && result_modifiert_release.is_ok() {
            Ok(())
        } else if result_key_release == Err(SubmitError::NotAlive)
            || result_modifiert_release == Err(SubmitError::NotAlive)
        {
            Err(SubmitError::NotAlive) // This error is more important because it will no longer be possible to send any keycodes
        } else {
            Err(SubmitError::InvalidKeycode)
        }
    }

    pub fn release_all_keys(&mut self) -> Result<(), SubmitError> {
        let pressed_keys: Vec<u32> = self.pressed_keys.iter().cloned().collect();
        let mut success = Ok(());
        for keycode in pressed_keys {
            if let Err(err) = self.send_keycode(&keycode.clone(), KeyMotion::Release) {
                success = Err(err); // Previous errors are disregarded
                error!(
                    "Failed to release all keys. Keycode causing the error: {}",
                    keycode
                );
            }
        }
        success
    }

    // Press and then release the key
    pub fn press_release_key(&mut self, keycode: &str) -> Result<(), SubmitError> {
        let press_result = self.send_key(keycode, KeyMotion::Press);
        if press_result.is_ok() {
            self.send_key(keycode, KeyMotion::Release)
        } else {
            press_result
        }
    }

    pub fn send_key(&mut self, keycode: &str, keymotion: KeyMotion) -> Result<(), SubmitError> {
        let keycode: String = keycode.to_ascii_uppercase(); // Necessary because all keycodes are uppercase
        if let Some(keycode) = input_event_codes_hashmap::KEY.get::<str>(&keycode) {
            self.send_keycode(keycode, keymotion)
        } else {
            error!("Keycode {} was invalid", keycode);
            Err(SubmitError::InvalidKeycode)
        }
    }

    pub fn toggle_key(&mut self, keycode: &str) -> Result<(), SubmitError> {
        let keycode: String = keycode.to_ascii_uppercase(); // Necessary because all keycodes are uppercase
        if let Some(keycode) = input_event_codes_hashmap::KEY.get::<str>(&keycode) {
            if self.pressed_keys.contains(keycode) {
                self.send_keycode(keycode, KeyMotion::Release)
            } else {
                self.send_keycode(keycode, KeyMotion::Press)
            }
        } else {
            error!("Keycode {} was invalid", keycode);
            Err(SubmitError::InvalidKeycode)
        }
    }

    pub fn send_keycode(&mut self, keycode: &u32, keymotion: KeyMotion) -> Result<(), SubmitError> {
        let time = self.get_time();
        if self.virtual_keyboard.as_ref().is_alive() {
            match keymotion {
                KeyMotion::Press => self.pressed_keys.insert(*keycode),
                KeyMotion::Release => self.pressed_keys.remove(keycode),
            };
            self.virtual_keyboard.key(time, *keycode, keymotion as u32);
            Ok(())
        } else {
            error!("Virtual_keyboard proxy was no longer alive");
            Err(SubmitError::NotAlive)
        }
    }

    pub fn release_all_modifiers(&mut self) -> Result<(), SubmitError> {
        let new_modifier_state = ModifiersBitflag::NO_MODIFIERS;
        self.send_modifiers_bitflag(new_modifier_state)
    }

    pub fn toggle_modifier(&mut self, modifier: keyboard::Modifier) -> Result<(), SubmitError> {
        let mut new_modifier_state = self.pressed_modifiers;
        new_modifier_state.toggle(ModifiersBitflag::from(modifier));
        self.send_modifiers_bitflag(new_modifier_state)
    }

    fn send_modifiers_bitflag(&mut self, modifiers: ModifiersBitflag) -> Result<(), SubmitError> {
        if self.virtual_keyboard.as_ref().is_alive() {
            self.virtual_keyboard.modifiers(
                modifiers.bits, //mods_depressed,
                0,              //mods_latched
                0,              //mods_locked
                0,              //group
            );
            self.pressed_modifiers = modifiers;
            Ok(())
        } else {
            error!("Virtual_keyboard proxy was no longer alive");
            Err(SubmitError::NotAlive)
        }
    }
}
