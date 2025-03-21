/// The definition for PCI byte of each frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciType {
    SingleFrame,      // SF: Single Frame (<= 7 bytes) - 0x00-0x0F
    FirstFrame,       // FF: First Frame (> 7 bytes) - 0x10-0x1F
    ConsecutiveFrame, // CF: Consecutive Frame (multi-frame) - 0x20-0x2F
    FlowControl,      // FC: Flow Control (to control CF transmission) 0x30-0x3F
}

#[derive(Debug, Clone, Copy)]
pub struct PciByte {
    pub pci_type: PciType,
    pub value: u8, // The actual PCI byte
}

impl From<PciByte> for u8 {
    fn from(pci_byte: PciByte) -> Self {
        match pci_byte.pci_type {
            PciType::SingleFrame => pci_byte.value & 0x0F,
            PciType::FirstFrame => 0x10 | (pci_byte.value & 0x0F),
            PciType::ConsecutiveFrame => 0x20 | (pci_byte.value & 0x0F),
            PciType::FlowControl => 0x30 | (pci_byte.value & 0x0F),
        }
    }
}

/// The implementation for PCI byte as ISO 15765-2
impl PciByte {
    pub fn new(pci_type: PciType, value: u8) -> Self {
        Self { pci_type, value }
    }

    pub fn get_type(&self) -> PciType {
        self.pci_type
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }

    pub fn as_byte(&self) -> u8 {
        match self.pci_type {
            PciType::SingleFrame => self.value & 0x0F,
            PciType::FirstFrame => 0x10 | (self.value & 0x0F),
            PciType::ConsecutiveFrame => 0x20 | (self.value & 0x0F),
            PciType::FlowControl => 0x30 | (self.value & 0x0F),
        }
    }
}
