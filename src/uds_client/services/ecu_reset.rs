//!  Provides methods to reset the ECU that includes soft-reset, hard-reset, ...
//!

use crate::uds_client::{DiagError, PciByte, UdsClient};
use automotive_diag::uds::UdsCommand;
use embedded_can::nb::Can;

/// Reset ECU subcommand
#[repr(u8)]
pub enum ResetType {
    RealTime = 0x40,
    Telematic = 0x41,
    Imx = 0x42,
    Esp32Wifi = 0x43,
    Esp32Ble = 0x44,
    Lte = 0x45,
    Lizard = 0x46,
    Cendric = 0x47,
}

impl From<ResetType> for u8 {
    fn from(reset_type: ResetType) -> Self {
        reset_type as u8
    }
}

impl TryFrom<i32> for ResetType {
    type Error = ();
    fn try_from(reset_type: i32) -> Result<Self, Self::Error> {
        match reset_type as u8 {
            0x40 => Ok(ResetType::RealTime),
            0x41 => Ok(ResetType::Telematic),
            0x42 => Ok(ResetType::Imx),
            0x43 => Ok(ResetType::Esp32Wifi),
            0x44 => Ok(ResetType::Esp32Ble),
            0x45 => Ok(ResetType::Lte),
            0x46 => Ok(ResetType::Lizard),
            0x47 => Ok(ResetType::Cendric),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
impl<T: Can> UdsClient<'_, T> {
    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x40
    /// Description:
    ///     The function will request an ECU reset event for Realtime chip.
    pub async fn uds_reset_118(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset 118");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::RealTime.into()],
        )
        .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x41
    /// Description:
    ///     The function will request an ECU reset event for Telematic chip.
    pub async fn uds_reset_148(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset 148");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::Telematic.into()],
        )
        .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x42
    /// Description:
    ///     The function will request an ECU reset event for IMX chip.
    pub async fn uds_reset_imx(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset imx");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(pci_byte, UdsCommand::ECUReset, &[ResetType::Imx.into()])
            .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x43
    /// Description:
    ///     The function will request an ECU reset event for Esp-Wifi chip.
    pub async fn uds_reset_esp32_wifi(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset esp-wifi");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::Esp32Wifi.into()],
        )
        .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x44
    /// Description:
    ///     The function will request an ECU reset event for Esp-Ble chip.
    pub async fn uds_reset_esp32_ble(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset esp-ble");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::Esp32Ble.into()],
        )
        .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x45
    /// Description:
    ///     The function will request an ECU reset event for LTE chip.
    pub async fn uds_reset_lte(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset lte");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(pci_byte, UdsCommand::ECUReset, &[ResetType::Lte.into()])
            .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x46
    /// Description:
    ///     The function will request an ECU reset event for Lizard chip.
    pub async fn uds_reset_lizard(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset lizard");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::Lizard.into()],
        )
        .await?;
        Ok(())
    }

    /// Service ID: 0x11 - ECU Reset
    ///     Sub-ID: 0x47
    /// Description:
    ///     The function will request an ECU reset event for Cendric chip.
    pub async fn uds_reset_cendric(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset cendric");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ECUReset,
            &[ResetType::Cendric.into()],
        )
        .await?;
        Ok(())
    }
}
