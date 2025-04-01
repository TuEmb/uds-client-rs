/// The definition for the Protocol Control Information (PCI) byte type used in ISO 15765-2 (CAN TP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciType {
    /// Single Frame (SF): Used for messages that fit in a single frame (<= 7 bytes).
    /// PCI byte range: 0x00-0x0F.
    SingleFrame,
    /// First Frame (FF): Indicates the start of a multi-frame message (> 7 bytes).
    /// PCI byte range: 0x10-0x1F.
    FirstFrame,
    /// Consecutive Frame (CF): Contains the continuation of a multi-frame message.
    /// PCI byte range: 0x20-0x2F.
    ConsecutiveFrame,
    /// Flow Control (FC): Used for flow control to regulate the transmission of consecutive frames.
    /// PCI byte range: 0x30-0x3F.
    FlowControl,
}

/// Represents a PCI byte, which consists of a PCI type and an associated value.
#[derive(Debug, Clone, Copy)]
pub struct PciByte {
    /// The type of PCI frame.
    pub pci_type: PciType,
    /// The actual PCI byte value.
    pub value: u8,
}

/// Implements conversion from `PciByte` to `u8`, encoding the PCI type and value into a single byte.
impl From<PciByte> for u8 {
    fn from(pci_byte: PciByte) -> Self {
        match pci_byte.pci_type {
            PciType::SingleFrame => pci_byte.value & 0x0F, // Mask with 0x0F for SF.
            PciType::FirstFrame => 0x10 | (pci_byte.value & 0x0F), // Set the first nibble to 0x1.
            PciType::ConsecutiveFrame => 0x20 | (pci_byte.value & 0x0F), // Set the first nibble to 0x2.
            PciType::FlowControl => 0x30 | (pci_byte.value & 0x0F), // Set the first nibble to 0x3.
        }
    }
}

/// Implementation for PCI byte handling based on ISO 15765-2 (CAN TP).
impl PciByte {
    /// Creates a new `PciByte` instance with the specified PCI type and value.
    pub fn new(pci_type: PciType, value: u8) -> Self {
        Self { pci_type, value }
    }

    /// Returns the PCI type of this byte.
    pub fn get_type(&self) -> PciType {
        self.pci_type
    }

    /// Returns the raw value of this PCI byte.
    pub fn get_value(&self) -> u8 {
        self.value
    }

    /// Encodes the PCI byte into a single `u8` value according to the CAN TP specification.
    pub fn as_byte(&self) -> u8 {
        match self.pci_type {
            PciType::SingleFrame => self.value & 0x0F, // Mask to get the lower 4 bits.
            PciType::FirstFrame => 0x10 | (self.value & 0x0F), // OR with 0x10 for FF.
            PciType::ConsecutiveFrame => 0x20 | (self.value & 0x0F), // OR with 0x20 for CF.
            PciType::FlowControl => 0x30 | (self.value & 0x0F), // OR with 0x30 for FC.
        }
    }
}
