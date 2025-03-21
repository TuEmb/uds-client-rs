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
    ECUError { code: u8, def: Option<String> },
    EmptyResponse,
    WrongMessage,
    ServerNotRunning,
    InvalidResponseLength,
    ParameterInvalid,
    ChannelError,
    HardwareError,
    NotImplemented(String),
    MismatchedIdentResponse { want: u16, received: u16 },
    Timeout,
}
