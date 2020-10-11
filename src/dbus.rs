use crate::user_interface;
use std::sync::{mpsc, mpsc::channel};
use std::sync::{Arc, Mutex};
mod dbus_client;
use dbus_client::DBusClient;
mod dbus_server;
use dbus_server::DBusServer;

pub struct DBusService {
    event_transmitter: mpsc::Sender<String>,
    visibility: Arc<Mutex<bool>>,
}

impl DBusService {
    pub fn new(sender: relm::Sender<user_interface::Msg>) -> DBusService {
        let visibility = Arc::new(Mutex::new(false));
        let visibility_clone = Arc::clone(&visibility); // Gets moved to DBusServer

        let (tx, rx) = channel(); // Create a simple streaming channel
        DBusClient::spawn_and_detach(rx);
        DBusServer::spawn_and_detach(Mutex::new(sender), visibility_clone);
        DBusService {
            event_transmitter: tx,
            visibility,
        }
    }

    pub fn change_visibility(&mut self, visible: bool) {
        *self.visibility.lock().unwrap() = visible;
        info!("Keyboard visibility changed to {}", visible);
    }

    pub fn haptic_feedback(&self, event: String) {
        match self.event_transmitter.send(event) {
            Ok(()) => info!(
                "Event was sent over the channel to the DBusClient"
            ),
            Err(_) => error!(
                "It was not possible to send the event over the channel to the DBusClient. The receiver was deallocated before."
            ),
        }
    }
}
