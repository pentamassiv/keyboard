use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use tempfile::tempfile;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{GlobalManager, Main, Proxy};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_v2::ZwpInputMethodV2;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

enum KeyMotion {
    Press = 1,
    Release = 0,
}

#[derive(Debug, Clone, Copy)]
enum ShiftState {
    Pressed = 1,
    Released = 0,
}

pub struct InputHandler {
    base_time: std::time::Instant,
    virtual_keyboard: Option<wayland_client::Main<ZwpVirtualKeyboardV1>>,
    shift_state: ShiftState,
    input_method: Option<wayland_client::Main<ZwpInputMethodV2>>,
}
impl InputHandler {
    fn enter_key(&self, keycode: u32, key_motion: u32) {
        if let Some(virtual_keyboard) = &self.virtual_keyboard {
            let duration = self.base_time.elapsed();
            let time = duration.as_millis();
            let time = time.try_into().unwrap();
            virtual_keyboard.key(time, keycode, key_motion);
        }
    }

    //pub fn send_str(&self, string: &str) {}
    //pub fn send_char(&self, character: char) {}

    //fn send_combo(&self, combo_vec: Vec<KeyCode>) {
    //Key::Physical(Physical::E),
    //Key::Unicode('c'),
    //Key::Unicode('h'),
    //Key::Unicode('o'),
    //}

    // Press and then release the key
    pub fn send_key(&mut self, keycode: &str) {
        if keycode == "Shift_L_upper" || keycode == "Shift_L_base" {
            println!("Shift");
            self.shift();
        } else if let Some(keycode) = input_event_codes_hashmap::KEY.get(keycode) {
            self.press_key(*keycode);
            self.release_key(*keycode);
        }
    }

    fn press_key(&self, keycode: u32) {
        self.enter_key(keycode, KeyMotion::Press as u32);
    }

    fn release_key(&self, keycode: u32) {
        self.enter_key(keycode, KeyMotion::Release as u32);
    }
    pub fn shift(&mut self) {
        if let Some(keyboard) = &self.virtual_keyboard {
            match self.shift_state {
                ShiftState::Pressed => self.shift_state = ShiftState::Released,
                ShiftState::Released => self.shift_state = ShiftState::Pressed,
            }
            keyboard.modifiers(
                self.shift_state as u32, //mods_depressed,
                0,                       //mods_latched
                0,                       //mods_locked
                0,                       //group
            )
        }
    }
}

fn get_wayland_input_handler(global_manager: &GlobalManager, seat: Proxy<WlSeat>) -> InputHandler {
    let virtual_kbd_mngr = global_manager
        .instantiate_exact::<ZwpVirtualKeyboardManagerV1>(1)
        .expect("Error: Your compositor does not understand the virtual_keyboard protocol!");
    //let input_method_mngr = global_manager
    //    .instantiate_exact::<ZwpInputMethodManagerV2>(1)
    //    .expect("Error: Your compositor does not understand the virtual_keyboard protocol!");
    let seat: WlSeat = WlSeat::from(seat);
    //let input_method = input_method_mngr.get_input_method(&seat);
    //input_method.commit_string("JOOOO!".to_string());
    //input_method.commit(0);
    //println!("Commit 0");
    //input_method.commit(1);
    //println!("Commit 1");
    let base_time = Instant::now();
    let shift_state = ShiftState::Released;
    InputHandler {
        base_time,
        virtual_keyboard: Some(virtual_keyboard),
        shift_state,
        input_method: None,
    }
}

pub fn init_virtual_keyboard(
    virtual_keyboard: Main<ZwpVirtualKeyboardV1>,
) -> Main<ZwpVirtualKeyboardV1> {
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
    virtual_keyboard.keymap(1, keymap_raw_fd, keymap_size_u32);
    virtual_keyboard
}
