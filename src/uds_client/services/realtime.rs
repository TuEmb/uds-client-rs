//!  Provides methods to reset the ECU that includes soft-reset, hard-reset, ...
//!

use crate::{
    socket_can::CanSocketTx,
    uds_client::{
        DiagError, PciByte, Response, UdsClient,
        frame::{UdsFlowControlFrame, UdsFrame},
    },
};
use automotive_diag::uds::UdsCommand;

/// Reset ECU subcommand
#[repr(u8)]
#[derive(Debug)]
pub enum RealTimeType {
    SlowRate = 0x01,   // 30 seconds
    MediumRate = 0x02, // 5 seconds
    FastRate = 0x03,   // 100ms
    Stop = 0x04,       // Stop sending
}

impl From<RealTimeType> for u8 {
    fn from(rt_type: RealTimeType) -> Self {
        rt_type as u8
    }
}

impl TryFrom<i32> for RealTimeType {
    type Error = ();
    fn try_from(rt_type: i32) -> Result<Self, Self::Error> {
        match rt_type as u8 {
            0x01 => Ok(RealTimeType::SlowRate),
            0x02 => Ok(RealTimeType::MediumRate),
            0x03 => Ok(RealTimeType::FastRate),
            0x04 => Ok(RealTimeType::Stop),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
impl<T: CanSocketTx> UdsClient<'_, T> {
    /// Service ID: 0x2A - Data Transmission
    ///     Sub-ID: 0x01
    /// Description:
    ///     The function will request an Realtime data sent from ECU with slow rate.
    pub async fn uds_real_time_data_slow(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send realtime data request (slow mode)");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 3);
        let re = self
            .send_command_with_response(
                pci_byte,
                UdsCommand::ReadDataByPeriodicIdentifier,
                &[0x01, 0xB0],
            )
            .await?;
        self.real_time_data_process(re).await?;
        Ok(())
    }

    /// Service ID: 0x2A - Data Transmission
    ///     Sub-ID: 0x02
    /// Description:
    ///     The function will request an Realtime data sent from ECU with medium rate.
    pub async fn uds_real_time_data_medium(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send realtime data request (medium mode)");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 3);
        let re = self
            .send_command_with_response(
                pci_byte,
                UdsCommand::ReadDataByPeriodicIdentifier,
                &[0x02, 0xB0],
            )
            .await?;
        self.real_time_data_process(re).await?;
        Ok(())
    }

    /// Service ID: 0x2A - Data Transmission
    ///     Sub-ID: 0x03
    /// Description:
    ///     The function will request an Realtime data sent from ECU with fast rate.
    pub async fn uds_real_time_data_fast(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send realtime data request (fast mode)");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 3);
        let re = self
            .send_command_with_response(
                pci_byte,
                UdsCommand::ReadDataByPeriodicIdentifier,
                &[0x03, 0xB0],
            )
            .await?;
        self.real_time_data_process(re).await?;
        Ok(())
    }

    /// Service ID: 0x2A - Data Transmission
    ///     Sub-ID: 0x04
    /// Description:
    ///     The function will send a stop event for realtime data from ECU.
    pub async fn uds_real_time_data_stop(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: stop realtime data");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 3);
        self.send_command_with_response(
            pci_byte,
            UdsCommand::ReadDataByPeriodicIdentifier,
            &[0x04, 0xB0],
        )
        .await?;
        Ok(())
    }

    /// Process the realtime data transfer from ECU
    async fn real_time_data_process(&mut self, response: UdsFrame) -> Result<(), DiagError> {
        let mut remain;
        if let UdsFrame::First(frame) = response {
            let flow_ctrl = UdsFlowControlFrame::new(0x00, 0x00, 0x7F, Vec::new()).unwrap();
            self.send_frame(UdsFrame::FlowControl(flow_ctrl)).await?;

            remain = frame.size as usize - frame.payload.len();
            let mut pre_idx = 0;
            while let Response::Ok(uds_frame) = self.receive().await {
                match uds_frame {
                    UdsFrame::Consecutive(frame) => {
                        remain -= frame.payload.len();
                        if frame.seq_num != if pre_idx == 15 { 0 } else { pre_idx + 1 } {
                            return Err(DiagError::InvalidResponseLength);
                        }
                        pre_idx = frame.seq_num;
                    }
                    UdsFrame::First(frame) => {
                        let flow_ctrl =
                            UdsFlowControlFrame::new(0x00, 0x00, 0x7F, Vec::new()).unwrap();
                        self.send_frame(UdsFrame::FlowControl(flow_ctrl)).await?;
                        remain = frame.size as usize - frame.payload.len();
                        pre_idx = 0;
                    }
                    _ => {}
                }
            }
        } else {
            return Err(DiagError::WrongMessage);
        }

        if remain == 0 {
            Ok(())
        } else {
            Err(DiagError::InvalidResponseLength)
        }
    }
}
