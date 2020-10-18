// Imports from other crates
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
    mpsc::channel,
    Arc, Mutex,
};

// Imports from other modules
use crate::user_interface;

// Modules
mod dbus_client;
mod dbus_server;
use dbus_client::DBusClient;
use dbus_server::DBusServer;

/// The DBusService starts the DBusClient and DBusServer and saves the state of the visibility of the keyboard.
/// It also can forward events that need to be sent to feedbackd to the DBusClient. This is used to give a haptic feedback when buttons are pressed and released
pub struct DBusService {
    event_transmitter: mpsc::Sender<String>,
    visibility: Arc<AtomicBool>,
}

impl DBusService {
    /// Starts the DBusClient and DBusServer and returns an DBusService to handle them.
    pub fn new(sender: relm::Sender<user_interface::Msg>) -> DBusService {
        let visibility = Arc::new(AtomicBool::new(false));
        let visibility_clone = Arc::clone(&visibility); // Gets moved to DBusServer

        let (tx, rx) = channel(); // Create a simple streaming channel
        DBusClient::spawn_and_detach(rx);
        DBusServer::spawn_and_detach(Mutex::new(sender), visibility_clone);
        DBusService {
            event_transmitter: tx,
            visibility,
        }
    }

    /// Changes the value of the visibility of the keyboard. It does not cause the keyboard to show or hide. That has to be done by the UI manager
    pub fn change_visibility(&mut self, visible: bool) {
        self.visibility.store(visible, Ordering::SeqCst);
        info!("Keyboard visibility changed to {}", visible);
    }

    /// Tell the DBusClient to send the event to feedbackd (used for haptic feedback)
    pub fn haptic_feedback(&self, event: String) {
        if let Ok(()) = self.event_transmitter.send(event) {
            info!("Event was sent over the channel to the DBusClient")
        } else {
            error!("It was not possible to send the event over the channel to the DBusClient. The receiver was deallocated before.")
        };
    }
}
