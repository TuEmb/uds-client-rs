//!  Provides methods to reset the ECU that includes soft-reset, hard-reset, ...
//!

use crate::{
    socket_can::CanSocketTx,
    uds_client::{DiagError, PciByte, UdsClient},
};
use automotive_diag::uds::UdsCommand;

#[allow(dead_code)]
impl<T: CanSocketTx> UdsClient<'_, T> {
    /// Service ID: 0x11 - ECU Reset
    /// Description:
    ///     The function will request an ECU reset event.
    pub async fn uds_reset_ecu(&mut self) -> Result<(), DiagError> {
        dbg!("UDS: send reset ECU");
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 2);
        self.send_command_with_response(pci_byte, UdsCommand::ECUReset, &[])
            .await?;
        Ok(())
    }
}
