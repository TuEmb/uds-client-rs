use uds_client_rs::{RealTimeType, ResetType};

impl TryFrom<i32> for UiEventTx {
    type Error = ();
    fn try_from(ui_type: i32) -> Result<Self, Self::Error> {
        match ui_type as u8 {
            0x01 => Ok(UiEventTx::RealTime(RealTimeType::SlowRate)),
            0x02 => Ok(UiEventTx::RealTime(RealTimeType::MediumRate)),
            0x03 => Ok(UiEventTx::RealTime(RealTimeType::FastRate)),
            0x04 => Ok(UiEventTx::RealTime(RealTimeType::Stop)),
            0x40 => Ok(UiEventTx::Reset(ResetType::RealTime)),
            0x41 => Ok(UiEventTx::Reset(ResetType::Telematic)),
            0x42 => Ok(UiEventTx::Reset(ResetType::Imx)),
            0x43 => Ok(UiEventTx::Reset(ResetType::Esp32Wifi)),
            0x44 => Ok(UiEventTx::Reset(ResetType::Esp32Ble)),
            0x45 => Ok(UiEventTx::Reset(ResetType::Lte)),
            0x46 => Ok(UiEventTx::Reset(ResetType::Lte)),
            0x47 => Ok(UiEventTx::Reset(ResetType::Cendric)),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum UiEventTx {
    Reset(ResetType),
    RealTime(RealTimeType),
}
