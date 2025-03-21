use std::{cell::RefCell, time::Duration};
use tokio::sync::{Mutex, Notify};

use super::DiagError;

#[derive(Debug, Clone, PartialEq)]
pub enum UdsResponse {
    SingleFrame(u8, u8, u8),              // (SID, SubID, Ident)
    FirstFrame(u16, u8, u8, u8, Vec<u8>), // (Size, DID, SubID, Ident, Data)
    ConsecutiveFrame(u8, Vec<u8>),        // (Index, Data)
    FlowControlFrame,                     // Not valid response
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Ok(UdsResponse),
    Error,
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
        Self(Mutex::new(RefCell::new(Response::Error)), Notify::new())
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
                Response::Error
            }
        }
    }

    /// Update the response data into response slot and raise a notification.
    pub async fn update_response(&self, new_data: Vec<u8>) -> Result<(), DiagError> {
        let res = self.process_response(new_data)?;
        self.0.lock().await.replace(Response::Ok(res)); // Lock and modify data
        self.1.notify_one(); // Notify the waiting thread
        Ok(())
    }

    fn process_response(&self, res: Vec<u8>) -> Result<UdsResponse, DiagError> {
        match res[0] & 0xF0 {
            0x00 => {
                // Single frame
                Ok(UdsResponse::SingleFrame(res[1], res[2], res[3]))
            }
            0x01 => {
                let size = (((res[0] & 0x0f) as u16) << 8) + res[1] as u16;
                // First frame
                Ok(UdsResponse::FirstFrame(
                    size,
                    res[2],
                    res[3],
                    res[4],
                    res[5..].to_vec(),
                ))
            }
            0x02 => {
                // Consecutive
                let idx = res[0] & 0x0F;
                Ok(UdsResponse::ConsecutiveFrame(idx, res[1..].to_vec()))
            }
            0x03 => {
                // Flow control frame
                Ok(UdsResponse::FlowControlFrame)
            }
            _ => Err(DiagError::NotSupported),
        }
    }
}
