use gdk_sys::{GdkDisplay, GdkSeat};
use glib::translate::ToGlibPtr;
use wayland_client::{
    protocol::wl_seat::WlSeat, sys::client::wl_display, Display, EventQueue, GlobalError,
    GlobalManager, Proxy,
};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;

// TODO check which modules need to be public
pub mod keymap;
pub mod layer_shell;
pub mod vk_service;

#[allow(non_camel_case_types)]
type wl_seat = libc::c_void;

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut GdkDisplay) -> *mut wl_display;
    fn gdk_wayland_seat_get_wl_seat(seat: *mut GdkSeat) -> *mut wl_seat;
}

type LayerShell = wayland_client::Main<ZwlrLayerShellV1>;
type VirtualKeyboardMgr = wayland_client::Main<ZwpVirtualKeyboardManagerV1>;
type InputMethodMgr = wayland_client::Main<ZwpInputMethodManagerV2>;

fn get_wl_display_seat() -> (Display, WlSeat) {
    let gdk_display = gdk::Display::get_default();
    let display_ptr = unsafe { gdk_wayland_display_get_wl_display(gdk_display.to_glib_none().0) };
    let display = unsafe { Display::from_external_display(display_ptr) };

    let gdk_seat = gdk_display.expect("No gdk_display").get_default_seat(); //.expect("No gdk_seat");
    let seat_ptr = unsafe { gdk_wayland_seat_get_wl_seat(gdk_seat.to_glib_none().0) };
    let seat = unsafe { Proxy::<WlSeat>::from_c_ptr(seat_ptr as *mut _) };
    let seat: WlSeat = WlSeat::from(seat);
    (display, seat)
}

fn get_wl_global_mgr(display: Display) -> (EventQueue, GlobalManager) {
    // Create the event queue
    let mut event_queue = display.create_event_queue();
    // Attach the display
    let attached_display = display.attach(event_queue.token());

    let global_mgr = GlobalManager::new(&attached_display);

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
            |_, _, _| println!("Event received that was not handled"), // For testing
                                                                       //|_, _, _| unreachable!(), // Original
        )
        .unwrap();
    (event_queue, global_mgr)
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
) -> (Option<VirtualKeyboardMgr>, Option<InputMethodMgr>) {
    let mut virtual_keyboard_option = None;
    let mut input_method_mgr_option = None;
    if let Ok(vk_mgr) = get_wl_vk_manager(global_mgr) {
        virtual_keyboard_option = Some(vk_mgr);
    } else {
        println!("Sorry, but your wayland compositor does not understand wp_virtual_keyboard. The keyboard is started with a label to enter the text");
    }
    if let Ok(im_mgr) = get_wl_im_manager(global_mgr) {
        input_method_mgr_option = Some(im_mgr);
    } else {
        println!("Sorry, but your wayland compositor does not understand wp_virtual_keyboard. The keyboard is started with a label to enter the text");
    }
    (virtual_keyboard_option, input_method_mgr_option)
}
pub fn get_layer_shell() -> Option<LayerShell> {
    let (display, _) = get_wl_display_seat();
    let (_, global_mgr) = get_wl_global_mgr(display); // Event queue can be dropped because it was only used to find out if layer_shell is available
    let mut layer_shell_option = None;
    if let Ok(layer_shell) = get_wl_layer_shell(&global_mgr) {
        layer_shell_option = Some(layer_shell);
    } else {
        println!("Sorry, but your wayland compositor does not understand gtk-layer-shell. The keyboard is started like a regular application")
    }
    layer_shell_option
}

pub fn init_wayland() -> (
    EventQueue,
    WlSeat,
    //Option<LayerShell>, // Possibly remove
    Option<VirtualKeyboardMgr>,
    Option<InputMethodMgr>,
) {
    let (display, seat) = get_wl_display_seat();
    let (event_queue, global_mgr) = get_wl_global_mgr(display);
    //let seat = get_wl_seat(&global_mgr);
    let (vk_mgr, im_mgr) = try_get_mgrs(&global_mgr); //let (layer_shell, vk_mgr, im_mgr) = try_get_mgrs(&global_mgr);
    (event_queue, seat, vk_mgr, im_mgr) //(seat, layer_shell, vk_mgr, im_mgr)
}
