#[derive(Debug)]
pub enum FrameError {
    InvalidFrameType,
    InvalidSize,
    InvalidCanLength,
    Others,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UdsFrame {
    Single(UdsSingleFrame),
    First(UdsFirstFrame),
    Consecutive(UdsConsecutiveFrame),
    FlowControl(UdsFlowControlFrame),
}

impl UdsFrame {
    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        match self {
            UdsFrame::Single(uds_single_frame) => uds_single_frame.to_vec(),
            UdsFrame::First(uds_first_frame) => uds_first_frame.to_vec(),
            UdsFrame::Consecutive(uds_consecutive_frame) => uds_consecutive_frame.to_vec(),
            UdsFrame::FlowControl(uds_flow_control_frame) => uds_flow_control_frame.to_vec(),
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, FrameError> {
        if data.is_empty() {
            return Err(FrameError::InvalidCanLength);
        }

        let frame_type = data[0] >> 4; // First 4 bits determine the frame type

        match frame_type {
            0x0 => {
                // Single Frame
                if data.len() < 2 {
                    return Err(FrameError::InvalidSize);
                }
                let size = data[0] & 0x0F;
                let sid = data[1];
                let did = if data.len() > 2 {
                    Some(((data[2] as u16) << 8) | data[3] as u16)
                } else {
                    None
                };
                let payload = data[2..].to_vec();
                Ok(UdsFrame::Single(UdsSingleFrame {
                    size,
                    sid,
                    did,
                    payload,
                }))
            }
            0x1 => {
                // First Frame
                if data.len() < 3 {
                    return Err(FrameError::InvalidSize);
                }
                let size = (((data[0] & 0x0F) as u16) << 8) | data[1] as u16;
                let sid = data[2];
                let did = if data.len() > 3 {
                    Some(((data[3] as u16) << 8) | data[4] as u16)
                } else {
                    None
                };
                let payload = data[3..].to_vec();
                Ok(UdsFrame::First(UdsFirstFrame {
                    size,
                    sid,
                    did,
                    payload,
                }))
            }
            0x2 => {
                // Consecutive Frame
                if data.len() < 2 {
                    return Err(FrameError::InvalidSize);
                }
                let seq_num = data[0] & 0x0F;
                let payload = data[1..].to_vec();
                Ok(UdsFrame::Consecutive(UdsConsecutiveFrame {
                    seq_num,
                    payload,
                }))
            }
            0x3 => {
                // Flow Control Frame
                if data.len() < 3 {
                    return Err(FrameError::InvalidSize);
                }
                let flag = data[0] & 0x0F;
                let block_size = data[1];
                let separation_time = data[2];
                let padding = data[3..].to_vec();
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

#[derive(Debug, Clone, PartialEq)]
pub struct UdsSingleFrame {
    pub size: u8,         // Size of payload (4 bits, max 7)
    pub sid: u8,          // Service Identifier (SID)
    pub did: Option<u16>, // Diagnostic Identifier (optional, some services use it)
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UdsFirstFrame {
    pub size: u16,        // Size of total payload (12 bits)
    pub sid: u8,          // Service Identifier (SID)
    pub did: Option<u16>, // Diagnostic Identifier
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UdsConsecutiveFrame {
    pub seq_num: u8, // Sequence number (0-15, 4 bits)
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UdsFlowControlFrame {
    pub flag: u8,            // Flag status (0=Continue, 1=Wait, 2=Overflow)
    pub block_size: u8,      // Number of Consecutive Frames to send before next FC
    pub separation_time: u8, // Delay between frames in milliseconds
    pub padding: Vec<u8>,
}

impl UdsSingleFrame {
    pub fn new(sid: u8, did: Option<u16>, payload: Vec<u8>) -> Result<Self, FrameError> {
        if payload.len() > 7 {
            return Err(FrameError::InvalidCanLength);
        }

        Ok(Self {
            size: payload.len() as u8,
            sid,
            did,
            payload,
        })
    }

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
    pub fn new(seq_num: u8, payload: Vec<u8>) -> Result<Self, FrameError> {
        if payload.len() > 7 {
            return Err(FrameError::InvalidCanLength);
        }

        Ok(Self { seq_num, payload })
    }

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

    pub fn to_vec(&self) -> Result<Vec<u8>, FrameError> {
        let frame = vec![
            0x30 | (self.flag & 0x0F), // PCI byte
            self.block_size,
            self.separation_time,
        ];

        Ok(frame)
    }
}
