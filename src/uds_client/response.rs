use std::{cell::RefCell, time::Duration};
use tokio::sync::{Mutex, Notify};

use super::{
    DiagError,
    frame::{FrameError, UdsFrame},
};

#[derive(Debug, Clone)]
pub enum Response {
    Ok(UdsFrame),
    Error(DiagError),
}

/// The response slot for each UDS request
pub struct ResponseSlot(pub Mutex<RefCell<Response>>, pub Notify);

impl Default for ResponseSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseSlot {
    // timeout in miliseconds
    const TIMEOUT: u64 = 1000;

    /// Create new response slot.
    pub fn new() -> Self {
        Self(
            Mutex::new(RefCell::new(Response::Error(DiagError::NotSupported))),
            Notify::new(),
        )
    }

    /// Get a response with blocking forever method.
    pub async fn get(&self) -> Result<Response, DiagError> {
        self.1.notified().await;
        let res = self.0.try_lock().unwrap().to_owned().into_inner();
        Ok(res)
    }

    /// Get a response with a <TIMEOUT> in ms.
    pub async fn wait_for_response(&self) -> Response {
        tokio::select! {
            _ = self.1.notified() => {
                let data = self.0.lock().await;
                data.borrow().clone()
            }
            _ = tokio::time::sleep(Duration::from_millis(Self::TIMEOUT)) => {
                Response::Error(DiagError::Timeout)
            }
        }
    }

    /// Update the response data into response slot and raise a notification.
    pub async fn update_response(&self, new_data: Vec<u8>) -> Result<(), FrameError> {
        let res = UdsFrame::from_vec(new_data)?;
        self.0.lock().await.replace(Response::Ok(res)); // Lock and modify data
        self.1.notify_one(); // Notify the waiting thread
        Ok(())
    }
}
