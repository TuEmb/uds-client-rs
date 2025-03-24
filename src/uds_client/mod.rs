mod client;
mod pci;
mod response;
mod services;

pub use client::UdsClient;
pub use pci::{PciByte, PciType};
pub use response::{Response, ResponseSlot};
pub use services::{RealTimeType, ResetType};

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
