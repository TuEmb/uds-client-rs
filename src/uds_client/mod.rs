mod client;
mod frame;
mod pci;
mod response;
mod services;

use std::fmt;

pub use client::UdsClient;
pub use frame::*;
pub use pci::{PciByte, PciType};
pub use response::{Response, ResponseSlot};
pub use services::RealTimeType;

#[derive(Clone, Debug)]
/// Diagnostic server error
pub enum DiagError {
    NotSupported,
    ECUError {
        /// Raw Negative response code from ECU
        code: u8,
        /// Negative response code definition according to protocol
        def: Option<String>,
    },
    /// Response empty
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    WrongMessage,
    /// Diagnostic server terminated!?
    ServerNotRunning,
    /// ECU Responded with a message, but the length was incorrect
    InvalidResponseLength,
    /// A parameter given to the function is invalid. Check the function's documentation
    /// for more information
    ParameterInvalid,
    /// Error with underlying communication channel
    ChannelError,
    /// Device hardware error
    HardwareError,
    /// Feauture is not iumplemented yet
    NotImplemented(String),
    /// Mismatched PID response ID
    MismatchedIdentResponse {
        /// Requested PID
        want: u16,
        /// Received PID from ECU
        received: u16,
    },
    /// timeout response
    Timeout,
}

impl fmt::Display for DiagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagError::NotSupported => write!(f, "Operation not supported"),
            DiagError::ECUError { code, def } => match def {
                Some(description) => write!(f, "ECU error: 0x{:02X} ({})", code, description),
                None => write!(f, "ECU error: 0x{:02X} (Unknown definition)", code),
            },
            DiagError::EmptyResponse => write!(f, "Response was empty"),
            DiagError::WrongMessage => write!(f, "ECU responded with an unrelated message"),
            DiagError::ServerNotRunning => write!(f, "Diagnostic server is not running"),
            DiagError::InvalidResponseLength => write!(f, "ECU responded with an incorrect message length"),
            DiagError::ParameterInvalid => write!(f, "Invalid parameter passed to function"),
            DiagError::ChannelError => write!(f, "Error in the communication channel"),
            DiagError::HardwareError => write!(f, "Hardware error detected"),
            DiagError::NotImplemented(feature) => write!(f, "Feature not implemented: {}", feature),
            DiagError::MismatchedIdentResponse { want, received } => write!(
                f,
                "Mismatched PID response: Expected 0x{:04X}, but received 0x{:04X}",
                want, received
            ),
            DiagError::Timeout => write!(f, "Response timeout"),
        }
    }
}
