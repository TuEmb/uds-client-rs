use std::{cell::RefCell, time::Duration};
use tokio::sync::{Mutex, Notify};

use super::{
    DiagError,
    frame::{FrameError, UdsFrame},
};

#[derive(Debug, Clone)]
pub enum Response {
    Ok(UdsFrame),     // Successful response with a UDS frame
    Error(DiagError), // Error response with a diagnostic error
}

/// The response slot for each UDS request
/// This struct holds the response data and a notification object to signal when the response is ready
pub struct ResponseSlot(pub Mutex<RefCell<Response>>, pub Notify, Duration);

impl Default for ResponseSlot {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ResponseSlot {
    /// Create a new ResponseSlot.
    ///
    /// This will initialize the slot with a default error (NotSupported) and set up the notification system.
    /// The `timeout_ms` is an optional input in milisecs, the default timeout is 1000ms.
    pub fn new(timeout_ms: Option<u64>) -> Self {
        Self(
            Mutex::new(RefCell::new(Response::Error(DiagError::NotSupported))), // Default to NotSupported error.
            Notify::new(), // Create a Notify object to handle asynchronous notifications.
            Duration::from_millis(timeout_ms.unwrap_or(1000)), // Use provided timeout or default to 1000ms.
        )
    }

    /// Get a response in a blocking manner. This will block forever until a response is available.
    ///
    /// It waits for the notification to be triggered and then locks the Mutex to retrieve the response.
    pub async fn get(&self) -> Result<Response, DiagError> {
        // Wait for the notification signal.
        self.1.notified().await;

        // Once notified, lock the Mutex and retrieve the response data.
        let res = self.0.try_lock().unwrap().to_owned().into_inner();

        // Return the response wrapped in Ok.
        Ok(res)
    }

    /// Get a response with a timeout. If no response is received within the timeout period, an error is returned.
    ///
    /// This function uses `tokio::select!` to wait for either the notification or the timeout.
    /// If the timeout expires, it returns a `Timeout` error.
    pub async fn wait_for_response(&self) -> Response {
        tokio::select! {
            // Wait for the notification that the response is available.
            _ = self.1.notified() => {
                // Lock the Mutex to access the response and return the data.
                let data = self.0.lock().await;
                data.borrow().clone()
            }
            // If the timeout period elapses, return a Timeout error response.
            _ = tokio::time::sleep(Duration::from_millis(self.2.as_millis() as u64)) => {
                Response::Error(DiagError::Timeout)
            }
        }
    }

    /// Update the response data in the response slot and notify the waiting task.
    ///
    /// This function is used to update the response after receiving new data.
    /// It creates a UdsFrame from the provided `new_data` and replaces the current response data.
    /// After updating, it notifies the waiting task that the response is ready.
    pub async fn update_response(&self, new_data: Vec<u8>) -> Result<(), FrameError> {
        // Convert the new data into a UdsFrame, handling any errors.
        let res = UdsFrame::from_vec(new_data)?;

        // Lock the Mutex and update the response with the new data (Ok variant).
        self.0.lock().await.replace(Response::Ok(res)); // Lock and modify data

        // Notify any waiting task that a response is available.
        self.1.notify_one(); // Notify the waiting thread

        // Return Ok if the update was successful.
        Ok(())
    }
}
