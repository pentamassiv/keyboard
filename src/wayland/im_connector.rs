use input_method_service::*;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

struct Connector {}

impl KeyboardVisability for Connector {
    fn show_keyboard(&self) {
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        println!("Hide keyboard");
    }
}

impl HintPurpose for Connector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}

fn new() -> Connector {
    let connector = Connector {};
    let (_display, seat, global_manager) = get_wayland_display_seat_globalmgr();
    let button = gtk::Button::with_label("Click me!");
    let im_manager = get_wayland_im_manager(&global_manager);
    let im_service = IMService::new(&seat, im_manager, connector);
    button.connect_clicked(move |_| {
        println!(
            "im_service is active: {}",
            im_service.borrow_mut().is_active()
        );
        let commit_string_result = im_service
            .borrow_mut()
            .commit_string(String::from("HelloWorld"));
        match commit_string_result {
            Ok(()) => println!("Successfully committed!"),
            _ => println!("Error when committing!"),
        }
        let commit_result = im_service.borrow_mut().commit();
        match commit_result {
            Ok(()) => println!("Successfully committed!"),
            _ => println!("Error when committing!"),
        }
    });
    connector
}
