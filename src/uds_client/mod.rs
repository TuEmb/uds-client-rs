mod client;
mod pci;
mod response;
mod services;

pub use client::UdsClient;
pub use pci::{PciByte, PciType};
pub use response::{Response, ResponseSlot};
pub use services::{RealTimeType, ResetType};

#[derive(Clone, Debug, thiserror::Error)]
/// Diagnostic server error
pub enum DiagError {
    /// The Diagnostic server does not support the request
    #[error("Diagnostic server does not support the request")]
    NotSupported,
    /// Diagnostic error code from the ECU itself
    #[error("ECU Negative response. Error 0x{:02X?}, definition: {:?}", code, def)]
    ECUError {
        /// Raw Negative response code from ECU
        code: u8,
        /// Negative response code definition according to protocol
        def: Option<String>,
    },
    /// Response empty
    #[error("ECU did not respond to the request")]
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    #[error("ECU response is out of order")]
    WrongMessage,
    /// Diagnostic server terminated!?
    #[error("Diagnostic server was terminated before the request")]
    ServerNotRunning,
    /// ECU Responded with a message, but the length was incorrect
    #[error("ECU response size was not the correct length")]
    InvalidResponseLength,
    /// A parameter given to the function is invalid. Check the function's documentation
    /// for more information
    #[error("Diagnostic function parameter invalid")]
    ParameterInvalid,
    /// Error with underlying communication channel
    #[error("Diagnostic server hardware channel error")]
    ChannelError,
    /// Device hardware error
    #[error("Diagnostic server hardware error")]
    HardwareError,
    /// Feauture is not iumplemented yet
    #[error("Diagnostic server feature is unimplemented: '{0}'")]
    NotImplemented(String),
    /// Mismatched PID response ID
    #[error(
        "Requested Ident 0x{:04X?}, but received ident 0x{:04X?}",
        want,
        received
    )]
    MismatchedIdentResponse {
        /// Requested PID
        want: u16,
        /// Received PID from ECU
        received: u16,
    },
    /// timeout response
    #[error("Diagnostic server doesn't response")]
    Timeout,
}
