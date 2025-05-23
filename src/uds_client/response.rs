use automotive_diag::uds::UdsError;
use std::{cell::RefCell, time::Duration};
use tokio::sync::{Mutex, Notify};

use super::{DiagError, frame::UdsFrame};

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
        let mut pending_response = None;
        loop {
            tokio::select! {
                _ = self.1.notified() => {
                    let data = self.0.lock().await;
                    match &*data.borrow() {
                        // handle the case where the response is a pending response
                        // and we need to wait for the next response or timeout
                        Response::Error(DiagError::ECUError { code, rsid: _, def: _ })
                            if *code == UdsError::RequestCorrectlyReceivedResponsePending =>
                        {
                            pending_response = Some(data.borrow().clone());
                            continue;
                        }
                        resp => return resp.clone(),
                    }
                }
                _ = tokio::time::sleep(self.2) => {
                    if let Some(pending_response) = pending_response {
                        return pending_response
                    } else {
                        return Response::Error(DiagError::Timeout)
                    }
                }
            }
        }
    }

    /// Update the response data in the response slot and notify the waiting task.
    ///
    /// This function is used to update the response after receiving new data.
    /// It creates a UdsFrame from the provided `new_data` and replaces the current response data.
    /// After updating, it notifies the waiting task that the response is ready.
    pub async fn update_response(&self, new_data: Vec<u8>) {
        // Convert the new data into a UdsFrame, handling any errors.
        let resp = match UdsFrame::from_vec(new_data) {
            Ok(frame) => Response::Ok(frame),
            Err(e) => Response::Error(e),
        };

        // Lock the Mutex and update the response with the new data.
        self.0.lock().await.replace(resp); // Lock and modify data

        // Notify any waiting task that a response is available.
        self.1.notify_one(); // Notify the waiting thread
    }
}
