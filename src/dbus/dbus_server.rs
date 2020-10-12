use crate::user_interface;
use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads};
use relm::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct DBusServer;
impl DBusServer {
    /// This functions spawns a new thread in which a new DBus connection is established.
    /// The new connection is used to allows for clients to change the keyboards visibility by calling the 'SetVisible' method or they can read the 'Visible' property
    pub fn spawn_and_detach(
        sender: Mutex<Sender<user_interface::Msg>>,
        visibility: Arc<AtomicBool>,
    ) {
        // Join handle is dropped because the new thread detaches itself from it when the handle is dropped and continues running
        // The handle is never used so its uneccessary
        thread::spawn(move || {
            // Starting up a connection to the session bus and requesting a name. It requests sm.puri.OSK0 because phosh is expecting
            // this name to change the visibility of the virtual keyboard
            let connection = Connection::new_session().unwrap();
            connection
                .request_name("sm.puri.OSK0", false, true, false)
                .unwrap();
            // Create a new crossroads instance.
            // The instance is configured so that introspection and properties interfaces
            // are added by default on object path additions.
            let mut crossroads = Crossroads::new();
            // Builds a new interface, which can be used for 'sm.puri.OSK0' objects.
            let iface_token = crossroads.register("sm.puri.OSK0", move |b| {
                // Adds the method SetVisible to the interface. This method allows clients to show or hide the keyboard over DBus.
                // Phosh uses this when you click on the little keyboard symbol in the bottom bar.
                // We have the method name, followed by names of input and output arguments (used for introspection).
                // The closure then controls the types of these arguments. The last argument to the closure is a tuple of the input arguments.
                b.method(
                    "SetVisible",
                    ("visible",),
                    (),
                    move |_ctx: &mut Context,
                          _visibility: &mut Arc<AtomicBool>,
                          (visible,): (bool,)| {
                        // This is what happens when the method is called.
                        info!(
                            "Dbus server received request to change visiblility to {}",
                            visible
                        );
                        // Sends the user_interface a message requesting to change the visibility
                        sender
                            .lock()
                            .unwrap()
                            .send(user_interface::Msg::Visible(visible))
                            .expect("send message");
                        Ok(())
                    },
                );
                // Adds the property 'Visible' to the interface. It is read only and tells the clients if the keyboard is currently visible or hidden
                b.property("Visible").get(|_, visibility| {
                    info!("Property 'Visible' was read");
                    Ok(visibility.load(Ordering::SeqCst))
                });
            });
            // Adds the '/sm/puri/OSK0' path, which implements the sm.puri.OSK0 interface,
            // to the crossroads instance.
            crossroads.insert("/sm/puri/OSK0", &[iface_token], visibility);
            info!("DBus server was spawned in a new thread and is ready to serve clients");
            // Serves clients forever
            crossroads.serve(&connection)
        });
    }
}
