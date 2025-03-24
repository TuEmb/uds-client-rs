//! # UDS Client for Rust (`uds-client-rs`)
//!
//! This crate provides a Unified Diagnostic Services (UDS) client for communication with automotive ECUs (Electronic Control Units).
//! It supports sending and receiving UDS messages over CAN using SocketCAN (Linux) and USB CAN adapters (Windows).
//!
//! ## Features
//! - Support for UDS over CAN (ISO 14229).
//! - Asynchronous API using `tokio`.
//! - Works with both Linux (`socketcan`) and Windows (`UsbCanSocket`).
//!
//! ## Running an Example
//!
//! To get started, you can run the provided example to test communication with an ECU.
//!
//! 1. **Choose the correct CAN interface:**
//!    - Linux: Use a `canX` interface (e.g., `can0`, `vcan0`).
//!    - Windows: Uses a supported USB-CAN adapter (Peak CAN).
//!
//! 2. **Run the example:**
//! ```sh
//! cd examples/uds_client_ui
//! cargo run --release
//! ```
//!
//! ## Usage
//!
//! Add `uds-client-rs` to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! uds-client-rs = "0.1"
//! ```
//!
//! Example usage:
//! ```rust
//! use uds_client_rs::UdsClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = UdsClient::new("can0", 0x784);
//!     if let Err(e) = client.uds_reset_118().await {
//!         eprintln!("Failed to request session: {:?}", e);
//!     }
//! }
//! ```
//!
//! ## License
//! This project is licensed under the MIT License.


mod socket_can;
mod uds_client;

pub use socket_can::*;
pub use uds_client::*;