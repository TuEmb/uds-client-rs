use crate::uds_client::{DiagError, PciByte, UdsClient};
use automotive_diag::uds::UdsCommand;
use embedded_can::nb::Can;
use log::{debug, warn};
use tokio::{fs::File, io::AsyncWriteExt};

#[allow(dead_code)]
impl<T: Can> UdsClient<'_, T> {
    /// Service ID: 0x36 - Transfer Data
    /// Description:
    ///     The function will request a data transfer from ECU.
    ///     The data will store in the <file> as raw binary
    pub async fn get_ecu_log(&mut self, mut file: File) -> Result<(), DiagError> {
        debug!("UDS: Request logging transfer");

        // send Single Frame for initializing request
        let pci_byte = PciByte::new(crate::uds_client::PciType::SingleFrame, 1);
        let response = self
            .send_command_with_response(pci_byte, UdsCommand::TransferData, &[])
            .await?;

        todo!();

        Ok(())
    }
}
