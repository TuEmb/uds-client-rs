//! # UDS Client
//!
//! ## Overview
//! The `UdsClient` provides an implementation for sending and receiving Unified Diagnostic Services (UDS) messages
//! over a CAN bus interface. It facilitates communication with ECUs (Electronic Control Units) in an automotive
//! network using UDS over CAN (ISO 14229).
//!
//! ## Features
//! - Supports **sending UDS requests** and **receiving responses** from an ECU.
//! - Implements **ISO-TP (ISO 15765-2) framing**, allowing for multi-frame messages.
//! - Handles **CAN IDs** (standard and extended) to ensure correct communication.
//! - Uses an **async-friendly design** to integrate with Rust's asynchronous runtime.
//!
//! ## Usage Example
//! ```rust
//! use embedded_can::{nb::Can, Frame, Id};
//! use uds_client::{UdsClient, DiagError, ResponseSlot};
//! use std::sync::Arc;
//!
//! async fn example_usage<T: Can>(channel: &mut T, resp_slot: &Arc<ResponseSlot>) -> Result<(), DiagError> {
//!     let id = Id::Extended(ExtendedId::new(0x7DF).unwrap());
//!     let mut client = UdsClient::new(channel, id, resp_slot);
//!
//!     // Example: Sending a diagnostic request
//!     client.send_command(0x10, &[0x01, 0x02, 0x03]).await?;
//!
//!     // Example: Receiving a response
//!     if let Ok(response) = client.receive().await {
//!         println!("Received response: {:?}", response);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Errors
//! The `UdsClient` may return the following errors:
//! - `DiagError::Timeout`: When a response is not received within the expected time.
//! - `DiagError::InvalidResponse`: When the received response does not match the expected UDS format.
//! - `DiagError::HardwareError`: When there is an issue with the CAN bus or adapter.
//!
//! ## Structs
//! - [`UdsClient`] - The main client struct for handling UDS communication.

use super::{DiagError, Response, ResponseSlot, response::UdsResponse};
use embedded_can::{ExtendedId, Frame, Id, nb::Can};
use log::debug;
use std::sync::{Arc, LazyLock};

pub struct UdsClient<'a, T: Can> {
    channel: &'a mut T,
    id: Id,
    resp: &'a Arc<ResponseSlot>,
}

#[allow(dead_code)]
impl<'a, T: Can> UdsClient<'a, T> {
    pub fn new(channel: &'a mut T, id: u32, resp: &'a LazyLock<Arc<ResponseSlot>>) -> Self {
        let id = Id::Extended(ExtendedId::new(id).unwrap());
        Self { channel, id, resp }
    }

    /// Send a command without the response.
    /// The frame includes <PCI> <CMD> <ARGS> as ISO 15765-2
    pub fn send_command<P: Into<u8>, M: Into<u8>>(
        &mut self,
        pci: P,
        cmd: M,
        args: &[u8],
    ) -> Result<(), DiagError> {
        let mut data = vec![pci.into(), cmd.into()];
        data.extend_from_slice(args);
        self.send_raw(&data)
    }

    /// Send a command with the response.
    /// The frame includes <PCI> <CMD> <ARGS> as ISO 15765-2
    pub async fn send_command_with_response<P: Into<u8>, M: Into<u8>>(
        &mut self,
        pci: P,
        cmd: M,
        args: &[u8],
    ) -> Result<UdsResponse, DiagError> {
        let mut data = vec![pci.into(), cmd.into()];
        data.extend_from_slice(args);
        match self.send_raw_with_response(&data).await? {
            Response::Ok(items) => {
                debug!("got response: {:?}", items);
                Ok(items)
            }
            Response::Error => Err(DiagError::Timeout),
        }
    }

    /// Internal function: send raw data as bytes array to CAN bus.
    fn send_raw(&mut self, data: &[u8]) -> Result<(), DiagError> {
        let frame = T::Frame::new(self.id, data).unwrap();
        println!("send raw data frame: {:?}", frame.data());
        self.channel.transmit(&frame).unwrap();
        Ok(())
    }

    /// Internal function: send raw data as bytes array to CAN bus and wait for a response.
    async fn send_raw_with_response(&mut self, data: &[u8]) -> Result<Response, DiagError> {
        let frame = T::Frame::new(self.id, data).unwrap();
        self.channel.transmit(&frame).unwrap();
        let response = self.resp.wait_for_response().await;
        Ok(response)
    }

    /// Receive the frame from UDS server
    pub async fn receive(&mut self) -> Result<Response, DiagError> {
        Ok(self.resp.wait_for_response().await)
    }

    // pub fn send_raw_frame_with_response(&mut self, frame: T::Frame) -> Result<(), DiagError> {
    //     if let Err(_) = self.channel.transmit(&frame) {
    //         return Err(DiagError::NotSupported);
    //     }
    //     Ok(())
    // }

    // pub fn wait_response() -> Result<(), DiagError> {
    //     Ok(())
    // }
}
