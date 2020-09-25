use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use tempfile::tempfile;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::Main;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

#[derive(Debug)]
pub enum KeyMotion {
    Press = 1,
    Release = 0,
}

#[derive(Debug, Clone, Copy)]
enum KeyState {
    Pressed = 1,
    Released = 0,
}

pub struct VKService {
    base_time: std::time::Instant,
    shift_state: KeyState,
    virtual_keyboard: Main<ZwpVirtualKeyboardV1>,
}

impl VKService {
    pub fn new(seat: &WlSeat, vk_mgr: Main<ZwpVirtualKeyboardManagerV1>) -> VKService {
        let base_time = Instant::now();
        let shift_state = KeyState::Released;
        let virtual_keyboard = vk_mgr.create_virtual_keyboard(&seat);
        let vk_service = VKService {
            base_time,
            shift_state,
            virtual_keyboard,
        };
        vk_service.init_virtual_keyboard();
        vk_service
    }

    fn init_virtual_keyboard(&self) {
        println!("keyboard initialized");
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
    }

    fn get_time(&self) -> u32 {
        let duration = self.base_time.elapsed();
        let time = duration.as_millis();
        time.try_into().unwrap()
    }

    // Press and then release the key
    pub fn submit_keycode(&mut self, keycode: &str) {
        if keycode == "Shift_L_upper" || keycode == "Shift_L_base" {
            println!("Shift");
            self.toggle_shift();
        } else {
            self.send_key(keycode, KeyMotion::Press);
            self.send_key(keycode, KeyMotion::Release);
        }
    }

    pub fn send_key(&self, keycode: &str, keymotion: KeyMotion) {
        if let Some(keycode) = input_event_codes_hashmap::KEY.get(keycode) {
            let time = self.get_time();
            println!("time: {}, keycode: {}", time, keycode);
            self.virtual_keyboard.key(time, *keycode, keymotion as u32);
        } else {
            println!("Not a valid keycode!")
        }
    }

    pub fn toggle_shift(&mut self) {
        match self.shift_state {
            KeyState::Pressed => self.shift_state = KeyState::Released,
            KeyState::Released => self.shift_state = KeyState::Pressed,
        }
        self.virtual_keyboard.modifiers(
            self.shift_state as u32, //mods_depressed,
            0,                       //mods_latched
            0,                       //mods_locked
            0,                       //group
        )
    }
}
