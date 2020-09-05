use gdk_sys::{GdkDisplay, GdkSeat};
use glib::translate::ToGlibPtr;
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use std::{
    fs::OpenOptions,
    io::{Seek, SeekFrom, Write},
};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::sys::client::wl_display;
use wayland_client::{Display, GlobalManager, Main, Proxy};

use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

pub mod keymap;

#[allow(non_camel_case_types)]
type wl_seat = libc::c_void;

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut GdkDisplay) -> *mut wl_display;
    fn gdk_wayland_seat_get_wl_seat(seat: *mut GdkSeat) -> *mut wl_seat;
}

pub fn init_wayland(window: &gtk::Window) -> InputHandler {
    let (global_manager, seat) = get_wayland_global_manager();
    let wayland_globals = get_wayland_globals(&global_manager);
    if let Some((_, _)) = wayland_globals.get("zwlr_layer_shell_v1") {
        let window_clone = window.clone();
        make_overlay_layer(window_clone);
    } else {
        println!("Sorry, but your wayland compositor does not understand gtk-layer-shell. The keyboard is started like a regular application")
    }
    if let Some((_, _)) = wayland_globals.get("zwp_virtual_keyboard_manager_v1") {
        get_wayland_input_handler(&global_manager, seat)
    } else {
        println!("Sorry, but your wayland compositor does not understand wp_virtual_keyboard. The keyboard is started with a label to enter the text");
        InputHandler {
            base_time: Instant::now(),
            virtual_keyboard: None,
        }
    }
}

fn make_overlay_layer(window: gtk::Window) {
    // Before the window is first realized, set it up to be a layer surface
    gtk_layer_shell::init_for_window(&window);

    // Order above normal windows
    gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);

    // The margins are the gaps around the window's edges
    // Margins and anchors can be set like this...
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 0);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 0);
    // ... or like this
    // Anchors are if the window is pinned to each edge of the output
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, false);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);
}

fn get_wayland_globals(global_manager: &GlobalManager) -> HashMap<String, (u32, u32)> {
    let mut globals = HashMap::new();
    // GlobalManager::list() provides a list of all globals advertized by the
    // server
    for (wayland_no, interface, version_no) in global_manager.list() {
        globals.insert(interface, (wayland_no, version_no));
    }
    globals
}
fn get_wayland_global_manager() -> (GlobalManager, Proxy<WlSeat>) {
    let gdk_display = gdk::Display::get_default();
    let wl_display_sys =
        unsafe { gdk_wayland_display_get_wl_display(gdk_display.to_glib_none().0) };
    let wl_display = unsafe { Display::from_external_display(wl_display_sys) };

    let gdk_seat = gdk_display.expect("No gdk_display").get_default_seat(); //.expect("No gdk_seat");
    let wl_seat_sys = unsafe { gdk_wayland_seat_get_wl_seat(gdk_seat.to_glib_none().0) };
    let wl_seat = unsafe { Proxy::<WlSeat>::from_c_ptr(wl_seat_sys as *mut _) };

    // Create the event queue
    let mut event_queue = wl_display.create_event_queue();
    // Attach the display
    let attached_display = wl_display.attach(event_queue.token());

    let global_manager = GlobalManager::new(&attached_display);

    // sync_roundtrip is a special kind of dispatching for the event queue.
    // Rather than just blocking once waiting for replies, it'll block
    // in a loop until the server has signalled that it has processed and
    // replied accordingly to all requests previously sent by the client.
    //
    // In our case, this allows us to be sure that after this call returns,
    // we have received the full list of globals.
    event_queue
        .sync_roundtrip(
            // we don't use a global state for this example
            &mut (),
            // The only object that can receive events is the WlRegistry, and the
            // GlobalManager already takes care of assigning it to a callback, so
            // we cannot receive orphan events at this point
            |_, _, _| unreachable!(),
        )
        .unwrap();
    (global_manager, wl_seat)
}
fn get_wayland_input_handler(global_manager: &GlobalManager, seat: Proxy<WlSeat>) -> InputHandler {
    let virtual_kbd_mngr = global_manager
        .instantiate_exact::<ZwpVirtualKeyboardManagerV1>(1)
        .expect("Error: Your compositor does not understand the virtual_keyboard protocol!");
    let seat: WlSeat = WlSeat::from(seat);
    let virtual_keyboard = virtual_kbd_mngr.create_virtual_keyboard(&seat);
    let virtual_keyboard = init_virtual_keyboard(virtual_keyboard);
    let base_time = Instant::now();
    InputHandler {
        base_time,
        virtual_keyboard: Some(virtual_keyboard),
    }
}
fn init_virtual_keyboard(
    virtual_keyboard: Main<ZwpVirtualKeyboardV1>,
) -> Main<ZwpVirtualKeyboardV1> {
    let src = keymap::KEYMAP;
    let keymap_size = keymap::KEYMAP.len();
    let keymap_size_u32: u32 = keymap_size.try_into().unwrap(); // Convert it from usize to u32, panics if it is not possible
    let keymap_size_u64: u64 = keymap_size.try_into().unwrap(); // Convert it from usize to u64, panics if it is not possible

    let mut keymap_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("keymap.mmap")
        .expect("Unable to open file");

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

enum KeyMotion {
    Press = 1,
    Release = 0,
}

pub struct InputHandler {
    base_time: std::time::Instant,
    virtual_keyboard: Option<wayland_client::Main<ZwpVirtualKeyboardV1>>,
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

    pub fn send_str(&self, string: &str) {}
    pub fn send_char(&self, character: char) {}

    //fn send_combo(&self, combo_vec: Vec<KeyCode>) {
    //Key::Physical(Physical::E),
    //Key::Unicode('c'),
    //Key::Unicode('h'),
    //Key::Unicode('o'),
    //}
    // Press and then release the key
    pub fn send_key(&self, keycode: &str) {
        if let Some(keycode) = input_event_codes_hashmap::KEY.get(keycode) {
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
}
