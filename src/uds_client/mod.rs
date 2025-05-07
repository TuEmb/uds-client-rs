mod client;
mod frame;
mod pci;
mod response;
mod services;

use automotive_diag::uds::{UdsCommand, UdsError};
pub use client::UdsClient;
pub use frame::*;
pub use pci::{PciByte, PciType};
pub use response::{Response, ResponseSlot};
pub use services::RealTimeType;

#[derive(Clone, Debug, thiserror::Error)]
/// Diagnostic server error
pub enum DiagError {
    #[error("Diagnostic server does not support the request")]
    NotSupported,
    /// Negative Response from ECU
    #[error("ECU error: 0x{:02X} ({:?})", *code as u8, def)]
    ECUError {
        /// Raw Negative response code from ECU
        code: UdsError,
        /// Requested SID
        rsid: UdsCommand,
        /// Negative response code definition according to protocol
        def: Option<String>,
    },
    /// Response empty
    #[error("ECU did not respond to the request")]
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    #[error("ECU response is wrong command. Expected: {want}, received {received}")]
    WrongMessage {
        /// Requested SID
        want: UdsCommand,
        /// Received SID from ECU
        received: UdsCommand,
    },
    /// ECU Responded wrong PCI type
    #[error("ECU response is wrong PCI type. Expected: {want:?}, received {received:?}")]
    WrongPciType {
        /// Requested SID
        want: PciType,
        /// Received SID from ECU
        received: PciType,
    },
    /// Diagnostic server terminated!?
    #[error("Diagnostic server was not running")]
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
    #[error("ECU server didn't response in time")]
    Timeout,
    /// Other Diagnostic Error
    #[error("Diag Frame Error: {error}")]
    FrameError { error: FrameError },
    /// Other Diagnostic Error
    #[error("Unkown Diagnostic Error")]
    Others,
}
