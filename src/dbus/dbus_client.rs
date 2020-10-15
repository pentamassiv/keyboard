use dbus::{arg::Variant, blocking::Connection};
use std::{collections::HashMap, sync::mpsc, thread, time::Duration};

pub struct DBusClient;
impl DBusClient {
    /// This functions spawns a new thread in which a new DBus connection is established.
    /// The new connection is used to send all events received over the receiver to feedbackd.
    pub fn spawn_and_detach(receiver: mpsc::Receiver<String>) {
        // Join handle is dropped because the new thread detaches itself from it when the handle is dropped and continues running
        // The handle is never used so its uneccessary
        thread::spawn(move || {
            let connection = Connection::new_session().unwrap(); // Starting up a connection to the session bus and requesting a name.
                                                                 // Creates the proxy for the object the events are sent to
            let proxy = connection.with_proxy(
                "org.sigxcpu.Feedback",
                "/org/sigxcpu/Feedback",
                Duration::from_millis(5000),
            );

            // This sets up the parameters for the method call on the proxy
            let app_id = "org.fingerboard.Feedback"; // ID of fingerboard, identifies the app requesting the event from feedbackd
            let timeout = -1; // Never timeout (This is only necessary if you want to end the feedback prematurely)
            let hints: HashMap<String, Variant<String>> = HashMap::new(); // No hints are sent
            info!("DBus client to handle haptic-feedback was spawned in a new thread and is waiting to receive events");

            // The DBusClient will wait (blocking) for events received over the channel.
            // Once an event is received, the 'TriggerFeedback' method is called and it waits for the next event.
            // This loop will end only if the channel is closed and should never happen.
            while let Ok(event) = receiver.recv() {
                info!("Received {} event to send to feedbackd", event);
                let (_event_id,): (u32,) = proxy
                    .method_call(
                        "org.sigxcpu.Feedback",
                        "TriggerFeedback",
                        (app_id, event, hints.clone(), timeout),
                    )
                    .unwrap();
            }
            error!("Channel to receive feedbackd events was closed");
        });
    }
}
