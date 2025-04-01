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

use crate::socket_can::CanSocketTx;

use super::{DiagError, Response, ResponseSlot, frame::UdsFrame};
use embedded_can::{ExtendedId, Frame, Id};
use log::debug;
use std::sync::{Arc, LazyLock};

pub struct UdsClient<'a, T: CanSocketTx> {
    channel: T,                  // The CAN socket channel to transmit data
    id: Id,                      // The identifier used for the CAN message
    resp: &'a Arc<ResponseSlot>, // A reference to the response slot for handling responses
}

#[allow(dead_code)]
impl<'a, T: CanSocketTx> UdsClient<'a, T> {
    /// Create a new UdsClient instance.
    ///
    /// Takes a CAN socket channel `channel`, a 32-bit identifier `id`, and a reference
    /// to a `ResponseSlot` wrapped in `Arc`. The `Id::Extended` is used to create a unique
    /// identifier for the CAN frame.
    pub fn new(channel: T, id: u32, resp: &'a LazyLock<Arc<ResponseSlot>>) -> Self {
        let id = Id::Extended(ExtendedId::new(id).unwrap());
        Self { channel, id, resp }
    }

    /// Send a command without expecting a response.
    ///
    /// This function sends a command using ISO 15765-2 format, which includes PCI, CMD,
    /// and ARGS. The `args` are added to the frame and sent using the `send_raw` method.
    pub async fn send_command<P: Into<u8>, M: Into<u8>>(
        &mut self,
        pci: P,
        cmd: M,
        args: &[u8],
    ) -> Result<(), DiagError> {
        let mut data = vec![pci.into(), cmd.into()];
        data.extend_from_slice(args);
        self.send_raw(&data).await
    }

    /// Send an UDS frame without expecting a response.
    ///
    /// This function sends the given `UdsFrame` to the CAN bus using the `send_raw` method
    /// after converting the frame into a byte vector.
    pub async fn send_frame(&mut self, frame: UdsFrame) -> Result<(), DiagError> {
        if let Ok(data) = frame.to_vec() {
            self.send_raw(&data).await
        } else {
            Err(DiagError::WrongMessage)
        }
    }

    /// Send an UDS frame and wait for a response.
    ///
    /// This function sends an `UdsFrame` to the CAN bus and waits for a response. If the
    /// response is valid (`Response::Ok`), it returns the response frame, otherwise returns
    /// the error contained in `Response::Error`.
    pub async fn send_frame_with_response(
        &mut self,
        frame: UdsFrame,
    ) -> Result<UdsFrame, DiagError> {
        if let Ok(data) = frame.to_vec() {
            match self.send_raw_with_response(&data).await? {
                Response::Ok(items) => {
                    debug!("got response: {:?}", items);
                    Ok(items)
                }
                Response::Error(e) => Err(e),
            }
        } else {
            Err(DiagError::WrongMessage)
        }
    }

    /// Send a command with the response.
    ///
    /// This function is similar to `send_command` but expects a response after sending
    /// the command. It returns the response frame (`UdsFrame`) if successful, or the
    /// error if something went wrong.
    pub async fn send_command_with_response<P: Into<u8>, M: Into<u8>>(
        &mut self,
        pci: P,
        cmd: M,
        args: &[u8],
    ) -> Result<UdsFrame, DiagError> {
        let mut data = vec![pci.into(), cmd.into()];
        data.extend_from_slice(args);
        match self.send_raw_with_response(&data).await? {
            Response::Ok(items) => {
                debug!("got response: {:?}", items);
                Ok(items)
            }
            Response::Error(e) => Err(e),
        }
    }

    /// Internal function: Send raw data to the CAN bus.
    ///
    /// This function sends the provided byte array `data` as a CAN frame using the `channel`.
    /// It creates a new `Frame` using the `id` and the data, and transmits it over the CAN bus.
    async fn send_raw(&mut self, data: &[u8]) -> Result<(), DiagError> {
        let frame = T::Frame::new(self.id, data).unwrap();
        println!("send raw data frame: {:?}", frame.data());
        self.channel.transmit(&frame).await.unwrap();
        Ok(())
    }

    /// Internal function: Send raw data to the CAN bus and wait for a response.
    ///
    /// This function sends the byte array `data` as a CAN frame and waits for a response using
    /// the `ResponseSlot`. It uses `wait_for_response` to receive the response, and returns the
    /// received `Response`.
    async fn send_raw_with_response(&mut self, data: &[u8]) -> Result<Response, DiagError> {
        let frame = T::Frame::new(self.id, data).unwrap();
        self.channel.transmit(&frame).await.unwrap();
        let response = self.resp.wait_for_response().await;
        Ok(response)
    }

    /// Receive a frame from the UDS server.
    ///
    /// This function waits for and receives a response from the UDS server using the `ResponseSlot`.
    /// It blocks until a response is available and returns the response.
    pub async fn receive(&mut self) -> Response {
        self.resp.wait_for_response().await
    }
}
