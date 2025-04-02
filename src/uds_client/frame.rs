use automotive_diag::uds::{UdsCommand, UdsError};

use super::PciType;

/// Represents errors that can occur while processing UDS frames.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FrameError {
    /// The frame type is not recognized.
    #[error("Invalid UDS frame type.")]
    InvalidFrameType,
    /// The frame size is invalid or too small.
    #[error("Frame size is incorrect.")]
    InvalidSize,
    /// The Service Identifier (SID) is invalid.
    #[error("Invalid Service Identifier (SID).")]
    InvalidSid,
    /// The Negative Response Code (NRC) is invalid.
    #[error("Invalid Negative Response Code (NRC).")]
    InvalidNrc,
    /// The CAN message length is invalid.
    #[error("Invalid CAN message length.")]
    InvalidCanLength,
    /// Other unspecified errors.
    #[error("An unknown error occurred.")]
    Others,
}

/// UDS frame types:
///     - Single Frame (SF)
///     - First Frame (FF)
///     - Consecutive Frame (CF)
///     - Flow Control Frame (FC)
///     - Negative Response Frame (NRC)
#[derive(Debug, Clone, PartialEq)]
pub enum UdsFrame {
    Single(UdsSingleFrame),
    First(UdsFirstFrame),
    Consecutive(UdsConsecutiveFrame),
    FlowControl(UdsFlowControlFrame),
    NegativeResp(UdsNegativeResponse),
}

impl UdsFrame {
    /// return PCI type of UDS frame
    pub fn pci_type(&self) -> PciType {
        match self {
            UdsFrame::Single(_) => PciType::SingleFrame,
            UdsFrame::First(_) => PciType::FirstFrame,
            UdsFrame::Consecutive(_) => PciType::ConsecutiveFrame,
            UdsFrame::FlowControl(_) => PciType::FlowControl,
            UdsFrame::NegativeResp(_) => PciType::SingleFrame,
        }
    }

    /// verify if the frame is negative response frame.
    pub fn is_negative_frame(&self) -> bool {
        matches!(self, UdsFrame::NegativeResp(_frame))
    }

    /// verify if the frame is single frame.
    pub fn is_single_frame(&self) -> bool {
        matches!(self, UdsFrame::Single(_frame))
    }

    /// verify if the frame is first frame.
    pub fn is_first_frame(&self) -> bool {
        matches!(self, UdsFrame::First(_frame))
    }

    /// verify if the frame is consecutive frame.
    pub fn is_consecutive_frame(&self) -> bool {
        matches!(self, UdsFrame::Consecutive(_frame))
    }

    /// verify if the frame is flow control frame.
    pub fn is_flow_control_frame(&self) -> bool {
        matches!(self, UdsFrame::FlowControl(_frame))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        match self {
            UdsFrame::Single(uds_single_frame) => uds_single_frame.to_vec(),
            UdsFrame::First(uds_first_frame) => uds_first_frame.to_vec(),
            UdsFrame::Consecutive(uds_consecutive_frame) => uds_consecutive_frame.to_vec(),
            UdsFrame::FlowControl(uds_flow_control_frame) => uds_flow_control_frame.to_vec(),
            UdsFrame::NegativeResp(uds_negative_response) => Ok(uds_negative_response.to_vec()),
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, FrameError> {
        let frame_type = data
            .first()
            .map(|b| b >> 4)
            .ok_or(FrameError::InvalidCanLength)?;

        match frame_type {
            0x0 => {
                // Single Frame
                let size = data[0] & 0x0F;
                let sid = *data.get(1).ok_or(FrameError::InvalidSize)?;

                if sid == 0x7F {
                    let rsid = UdsCommand::from_repr(*data.get(2).ok_or(FrameError::InvalidSid)?)
                        .ok_or(FrameError::InvalidSid)?;
                    let nrc = UdsError::from_repr(*data.get(3).ok_or(FrameError::InvalidNrc)?)
                        .ok_or(FrameError::InvalidNrc)?;
                    return Ok(UdsFrame::NegativeResp(UdsNegativeResponse {
                        size,
                        rsid,
                        nrc,
                    }));
                }

                let did = if data.len() > 2 {
                    Some(u16::from_be_bytes([data[2], *data.get(3).unwrap_or(&0)]))
                } else {
                    None
                };

                let payload_start = if did.is_some() { 4 } else { 2 };
                let payload = data.get(payload_start..).unwrap_or(&[]).to_vec();

                Ok(UdsFrame::Single(UdsSingleFrame {
                    size,
                    sid,
                    did,
                    payload,
                }))
            }
            0x1 => {
                // First Frame
                let size = (((data[0] & 0x0F) as u16) << 8)
                    | (*data.get(1).ok_or(FrameError::InvalidSize)? as u16);
                let sid = *data.get(2).ok_or(FrameError::InvalidSize)?;

                let did = data
                    .get(3..5)
                    .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
                let payload_start = if did.is_some() { 5 } else { 3 };
                let payload = data.get(payload_start..).unwrap_or(&[]).to_vec();

                Ok(UdsFrame::First(UdsFirstFrame {
                    size,
                    sid,
                    did,
                    payload,
                }))
            }
            0x2 => {
                // Consecutive Frame
                let seq_num = data[0] & 0x0F;
                let payload = data.get(1..).unwrap_or(&[]).to_vec();
                Ok(UdsFrame::Consecutive(UdsConsecutiveFrame {
                    seq_num,
                    payload,
                }))
            }
            0x3 => {
                // Flow Control Frame
                let (flag, block_size, separation_time) = (
                    data[0] & 0x0F,
                    *data.get(1).ok_or(FrameError::InvalidSize)?,
                    *data.get(2).ok_or(FrameError::InvalidSize)?,
                );
                let padding = data.get(3..).unwrap_or(&[]).to_vec();
                Ok(UdsFrame::FlowControl(UdsFlowControlFrame {
                    flag,
                    block_size,
                    separation_time,
                    padding,
                }))
            }
            _ => Err(FrameError::InvalidFrameType),
        }
    }
}

/// Represents a UDS Negative Response frame.
/// This frame is sent by the ECU when a UDS request fails.
#[derive(Debug, Clone, PartialEq)]
pub struct UdsNegativeResponse {
    /// Size of the payload (only 4 bits are used, max value is 7).
    pub size: u8,
    /// Service Identifier (SID) that caused the negative response.
    pub rsid: UdsCommand,
    /// Negative Response Code (NRC) indicating the reason for failure.
    pub nrc: UdsError,
}

/// Represents a UDS Single Frame.
/// This frame is used when the total payload fits within a single CAN frame.
#[derive(Debug, Clone, PartialEq)]
pub struct UdsSingleFrame {
    /// Size of the payload (only 4 bits are used, max value is 7).
    pub size: u8,
    /// Service Identifier (SID) for the request or response.
    pub sid: u8,
    /// Optional Diagnostic Identifier (DID), used in certain services.
    pub did: Option<u16>,
    /// The actual payload data for the request or response.
    pub payload: Vec<u8>,
}

/// Represents a UDS First Frame.
/// This frame is sent when the payload is too large for a single frame.
/// It contains the total size of the payload and the initial data.
#[derive(Debug, Clone, PartialEq)]
pub struct UdsFirstFrame {
    /// Total size of the payload (only 12 bits are used).
    pub size: u16,
    /// Service Identifier (SID) for the request or response.
    pub sid: u8,
    /// Optional Diagnostic Identifier (DID), used in certain services.
    pub did: Option<u16>,
    /// The first portion of the payload.
    pub payload: Vec<u8>,
}

/// Represents a UDS Consecutive Frame.
/// This frame is used for multi-frame transmissions after the First Frame.
#[derive(Debug, Clone, PartialEq)]
pub struct UdsConsecutiveFrame {
    /// Sequence number (4 bits, values range from 0 to 15).
    pub seq_num: u8,
    /// The next portion of the payload.
    pub payload: Vec<u8>,
}

/// Represents a UDS Flow Control Frame.
/// This frame is sent by the receiver to control the flow of multi-frame transmissions.
#[derive(Debug, Clone, PartialEq)]
pub struct UdsFlowControlFrame {
    /// Flow control flag:
    /// - `0x00` = Continue to send (CTS)
    /// - `0x01` = Wait (WT)
    /// - `0x02` = Overflow/abort (OVFLW)
    pub flag: u8,
    /// The number of Consecutive Frames the sender can transmit before waiting.
    pub block_size: u8,
    /// Minimum separation time (ST) in milliseconds between transmitted frames.
    pub separation_time: u8,
    /// Optional padding bytes (if required for 8-byte CAN frames).
    pub padding: Vec<u8>,
}

impl UdsNegativeResponse {
    /// Creates a new UDS Negative Response frame.
    ///
    /// # Parameters:
    /// - `rsid`: The requested service identifier that failed.
    /// - `nrc`: The negative response code indicating the failure reason.
    /// - `size`: Size of the response payload (max 7).
    ///
    /// # Returns:
    /// - A `UdsNegativeResponse` instance.
    pub fn new(rsid: UdsCommand, nrc: UdsError, size: u8) -> Self {
        Self { size, rsid, nrc }
    }

    /// Converts the negative response frame into a CAN frame byte vector.
    ///
    /// # Returns:
    /// - `Vec<u8>`: A byte array representing the negative response frame.
    pub fn to_vec(&self) -> Vec<u8> {
        vec![self.size & 0x0F, 0x7F, self.rsid.into(), self.nrc.into()]
    }
}

impl UdsSingleFrame {
    /// Creates a new UDS Single Frame.
    ///
    /// # Parameters:
    /// - `sid`: Service Identifier.
    /// - `did`: Optional Diagnostic Identifier.
    /// - `payload`: The payload data (max 7 bytes).
    ///
    /// # Returns:
    /// - `Ok(UdsSingleFrame)`: If the payload size is valid.
    /// - `Err(FrameError)`: If the payload exceeds 7 bytes.
    pub fn new(sid: u8, did: Option<u16>, payload: Vec<u8>) -> Result<Self, FrameError> {
        if payload.len() > 7 {
            return Err(FrameError::InvalidCanLength);
        }

        let size = if did.is_some() {
            payload.len() + 3
        } else {
            payload.len() + 1
        } as u8;

        Ok(Self {
            size,
            sid,
            did,
            payload,
        })
    }

    /// Converts the single frame into a CAN frame byte vector.
    ///
    /// # Returns:
    /// - `Ok(Vec<u8>)`: The CAN frame representation.
    /// - `Err(FrameError)`: If the payload size exceeds 7 bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        if self.payload.len() > 7 {
            return Err(FrameError::InvalidSize);
        }

        let mut frame = Vec::new();
        frame.push(self.size & 0x0F); // PCI byte (first nibble is 0 for Single Frame)
        frame.push(self.sid);
        if let Some(did) = self.did {
            frame.extend_from_slice(&did.to_be_bytes());
        }
        frame.extend_from_slice(&self.payload);

        Ok(frame)
    }
}

impl UdsFirstFrame {
    /// Creates a new UDS First Frame for multi-frame communication.
    ///
    /// # Parameters:
    /// - `sid`: Service Identifier.
    /// - `size`: Total payload size.
    /// - `did`: Optional Diagnostic Identifier.
    /// - `payload`: Initial chunk of the payload (max 6 bytes).
    ///
    /// # Returns:
    /// - `Ok(UdsFirstFrame)`: If the payload size is valid.
    /// - `Err(FrameError)`: If the payload exceeds 6 bytes.
    pub fn new(sid: u8, size: u16, did: Option<u16>, payload: Vec<u8>) -> Result<Self, FrameError> {
        if payload.len() > 6 {
            return Err(FrameError::InvalidCanLength);
        }

        Ok(Self {
            size,
            sid,
            did,
            payload,
        })
    }

    /// Converts the first frame into a CAN frame byte vector.
    ///
    /// # Returns:
    /// - `Ok(Vec<u8>)`: The CAN frame representation.
    /// - `Err(FrameError)`: If the payload size exceeds 6 bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        if self.payload.len() > 6 {
            return Err(FrameError::InvalidSize);
        }

        let mut frame = Vec::new();
        frame.push(0x10 | ((self.size >> 8) as u8 & 0x0F)); // PCI first byte
        frame.push((self.size & 0xFF) as u8); // PCI second byte
        frame.push(self.sid);
        if let Some(did) = self.did {
            frame.extend_from_slice(&did.to_be_bytes());
        }
        frame.extend_from_slice(&self.payload);

        Ok(frame)
    }
}

impl UdsConsecutiveFrame {
    /// Creates a new UDS Consecutive Frame.
    ///
    /// # Parameters:
    /// - `seq_num`: Sequence number (0-15).
    /// - `payload`: Payload data (max 7 bytes).
    ///
    /// # Returns:
    /// - `Ok(UdsConsecutiveFrame)`: If the payload size is valid.
    /// - `Err(FrameError)`: If the payload exceeds 7 bytes.
    pub fn new(seq_num: u8, payload: Vec<u8>) -> Result<Self, FrameError> {
        if payload.len() > 7 {
            return Err(FrameError::InvalidCanLength);
        }

        Ok(Self { seq_num, payload })
    }

    /// Converts the consecutive frame into a CAN frame byte vector.
    ///
    /// # Returns:
    /// - `Ok(Vec<u8>)`: The CAN frame representation.
    /// - `Err(FrameError)`: If the payload size exceeds 7 bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        if self.payload.len() > 7 {
            return Err(FrameError::InvalidSize);
        }

        let mut frame = Vec::new();
        frame.push(0x20 | (self.seq_num & 0x0F)); // PCI byte
        frame.extend_from_slice(&self.payload);

        Ok(frame)
    }
}

impl UdsFlowControlFrame {
    /// Creates a new UDS Flow Control Frame.
    ///
    /// # Parameters:
    /// - `flag`: Flow control flag (0=CTS, 1=Wait, 2=Overflow).
    /// - `block_size`: Number of consecutive frames before next flow control.
    /// - `separation_time`: Time delay (in ms) between frames.
    /// - `padding`: Optional padding data (max 5 bytes).
    ///
    /// # Returns:
    /// - `Ok(UdsFlowControlFrame)`: If the padding size is valid.
    /// - `Err(FrameError)`: If the padding exceeds 5 bytes.
    pub fn new(
        flag: u8,
        block_size: u8,
        separation_time: u8,
        padding: Vec<u8>,
    ) -> Result<Self, FrameError> {
        if padding.len() > 5 {
            return Err(FrameError::InvalidCanLength);
        }
        Ok(Self {
            flag,
            block_size,
            separation_time,
            padding,
        })
    }

    /// Converts the flow control frame into a CAN frame byte vector.
    ///
    /// # Returns:
    /// - `Ok(Vec<u8>)`: The CAN frame representation.
    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        let mut frame = vec![
            0x30 | (self.flag & 0x0F), // PCI byte
            self.block_size,
            self.separation_time,
        ];

        // Append padding if any
        frame.extend_from_slice(&self.padding);

        Ok(frame)
    }
}
