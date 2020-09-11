use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Display, GlobalError, GlobalManager, Main};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;

use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;

pub mod keymap;
pub mod layer_shell;
pub mod submitter;
mod vk_service;

fn get_wl_display_global_mgr() -> (Display, GlobalManager) {
    let display = Display::connect_to_name("wayland-0").unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());
    let global_mgr = GlobalManager::new(&attached_display);

    // Make a synchronized roundtrip to the wayland server.
    //
    // When this returns it must be true that the server has already
    // sent us all available globals.
    event_queue
        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
        .unwrap();
    global_mgr;
    (display, global_mgr)
}

fn get_wl_seat(global_mgr: GlobalManager) -> Main<WlSeat> {
    global_mgr
        .instantiate_exact::<WlSeat>(1)
        .expect("Could not get a seat!")
}

fn get_wl_im_manager(
    global_mgr: &GlobalManager,
) -> Result<wayland_client::Main<ZwpInputMethodManagerV2>, GlobalError> {
    global_mgr.instantiate_exact::<ZwpInputMethodManagerV2>(1)
    //.expect("Error: Your compositor does not understand the virtual_keyboard protocol!")
}

fn get_wl_vk_manager(
    global_mgr: &GlobalManager,
) -> Result<wayland_client::Main<ZwpVirtualKeyboardManagerV1>, GlobalError> {
    global_mgr.instantiate_exact::<ZwpVirtualKeyboardManagerV1>(1)
    //.expect("Error: Your compositor does not understand the virtual_keyboard protocol!")
}
fn get_wl_layer_shell(
    global_mgr: &GlobalManager,
) -> Result<wayland_client::Main<ZwlrLayerShellV1>, GlobalError> {
    global_mgr.instantiate_exact::<ZwlrLayerShellV1>(1)
    //.expect("Error: Your compositor does not understand the layer-shell protocol!")
}

fn try_get_mgrs(
    global_mgr: &GlobalManager,
    seat: &WlSeat,
    //window: &gtk::Window,
) -> (
    Option<wayland_client::Main<ZwlrLayerShellV1>>,
    Option<wayland_client::Main<ZwpVirtualKeyboardManagerV1>>,
    Option<wayland_client::Main<ZwpInputMethodManagerV2>>,
) {
    let layer_shell_option = None;
    let virtual_keyboard_option = None;
    let input_method_mgr_option = None;
    if let Ok(layer_shell) = get_wl_layer_shell(global_mgr) {
        let layer_shell_option = Some(layer_shell);
    //let window_clone = window.clone();
    //layer_shell::make_overlay_layer(window_clone);
    } else {
        println!("Sorry, but your wayland compositor does not understand gtk-layer-shell. The keyboard is started like a regular application")
    }
    if let Ok(vk_mgr) = get_wl_vk_manager(global_mgr) {
        let virtual_keyboard = vk_mgr.create_virtual_keyboard(seat);
        let virtual_keyboard_option = Some(virtual_keyboard);
    } else {
        println!("Sorry, but your wayland compositor does not understand wp_virtual_keyboard. The keyboard is started with a label to enter the text");
    }
    if let Ok(im_mgr) = get_wl_im_manager(global_mgr) {
        let input_method_mgr_option = Some(im_mgr);
    } else {
        println!("Sorry, but your wayland compositor does not understand wp_virtual_keyboard. The keyboard is started with a label to enter the text");
    }
    (
        layer_shell_option,
        virtual_keyboard_option,
        input_method_mgr_option,
    )
}

pub fn init_wayland() -> (
    Main<WlSeat>,
    Option<wayland_client::Main<ZwlrLayerShellV1>>,
    Option<wayland_client::Main<ZwpVirtualKeyboardManagerV1>>,
    Option<wayland_client::Main<ZwpInputMethodManagerV2>>,
) {
    let (display, global_mgr) = get_wl_display_global_mgr();
    let seat = get_wl_seat(global_mgr);
    let (layer_shell, vk_mgr, im_mgr) = try_get_mgrs(&global_mgr, &seat);
    (seat, layer_shell, vk_mgr, im_mgr)
}
