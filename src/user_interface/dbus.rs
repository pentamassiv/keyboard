use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads};
use relm::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(feature = "haptic-feedback")]
use dbus::arg::Variant;
#[cfg(feature = "haptic-feedback")]
use std::collections::HashMap;
#[cfg(feature = "haptic-feedback")]
use std::time::Duration;

struct Visibility {
    is_visible: bool,
}

pub struct DBusService {
    #[cfg(feature = "haptic-feedback")]
    client: DBusClient,
    server_running: bool,
    visibility: Arc<Mutex<Visibility>>,
}

#[cfg(feature = "haptic-feedback")]
struct DBusClient {
    connection: Connection,
}

#[cfg(feature = "haptic-feedback")]
impl DBusClient {
    fn new(connection: Connection) -> DBusClient {
        DBusClient { connection }
    }
    fn send_event(&self, event: String) {
        let proxy = self.connection.with_proxy(
            "org.sigxcpu.Feedback",
            "/org/sigxcpu/Feedback",
            Duration::from_millis(5000),
        );
        let app_id = "Dbus.client.test";
        let timeout = -1;
        let hints: HashMap<String, Variant<String>> = HashMap::new();
        info!("Sent {} event to feedbackd", event);
        let (event_id,): (u32,) = proxy
            .method_call(
                "org.sigxcpu.Feedback",
                "TriggerFeedback",
                (app_id, event, hints, timeout),
            )
            .unwrap();
    }
}

impl DBusService {
    pub fn new(sender: Sender<super::Msg>) -> Option<DBusService> {
        // Let's start by starting up a connection to the session bus and request a name.
        let server_connection = Connection::new_session().unwrap();
        #[cfg(feature = "haptic-feedback")]
        let client_connection = Connection::new_session().unwrap();
        #[cfg(feature = "haptic-feedback")]
        let client = DBusClient::new(client_connection);
        let visibility = Arc::new(Mutex::new(Visibility { is_visible: false }));
        let visibility_clone = Arc::clone(&visibility);
        let mut dbus_service = DBusService {
            #[cfg(feature = "haptic-feedback")]
            client,
            visibility,
            server_running: false,
        };
        dbus_service.spawn_server_and_detach(
            Mutex::new(sender),
            server_connection,
            visibility_clone,
        );
        Some(dbus_service)
    }
    pub fn change_visibility(&mut self, visible: bool) {
        self.visibility.lock().unwrap().is_visible = visible;
        info!("Keyboard visibility changed to {}", visible);
    }

    #[cfg(feature = "haptic-feedback")]
    pub fn haptic_feedback(&self, event: String) {
        self.client.send_event(event);
    }

    fn spawn_server_and_detach(
        &mut self,
        sender: Mutex<Sender<super::Msg>>,
        connection: Connection,
        visibility: Arc<Mutex<Visibility>>,
    ) {
        if !self.server_running {
            connection
                .request_name("sm.puri.OSK0", false, true, false)
                .unwrap();
            // Create a new crossroads instance.
            // The instance is configured so that introspection and properties interfaces
            // are added by default on object path additions.
            let mut crossroads = Crossroads::new();
            // Builds a new interface, which can be used for "Hello" objects.
            let iface_token = crossroads.register("sm.puri.OSK0", move |b| {
                // Adds a method to the interface. We have the method name, followed by
                // names of input and output arguments (used for introspection). The closure then controls
                // the types of these arguments. The last argument to the closure is a tuple of the input arguments.
                b.method(
                    "SetVisible",
                    ("visible",),
                    (),
                    move |ctx: &mut Context,
                          visibility: &mut Arc<Mutex<Visibility>>,
                          (visible,): (bool,)| {
                        // And here's what happens when the method is called.
                        info!(
                            "Dbus server received request to change visiblility to {}",
                            visible
                        );
                        sender
                            .lock()
                            .unwrap()
                            .send(super::Msg::Visible(visible))
                            .expect("send message");
                        // And the return value is a tuple of the output arguments.
                        Ok(())
                    },
                );
                // The "Visible" property is read only
                b.property("Visible").get(|_, visibility| {
                    info!("Property 'Visible' was read");
                    Ok(visibility.lock().unwrap().is_visible)
                });
            });
            // Let's add the "/sm/puri/OSK0" path, which implements the com.example.dbustest interface,
            // to the crossroads instance.
            crossroads.insert("/sm/puri/OSK0", &[iface_token], visibility);
            self.start_server(connection, crossroads);
        }
    }

    fn start_server(&mut self, connection: Connection, crossroads: Crossroads) {
        // Join handle is dropped because the new thread detaches itself from it when the handle is dropped and continues running
        // The handle is never used so its uneccessary
        self.server_running = true;
        info!("DBus server was started");
        thread::spawn(move || crossroads.serve(&connection));
    }
}
