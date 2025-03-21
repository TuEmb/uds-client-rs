use crate::uds_client::{DiagError, UdsClient};
use embedded_can::nb::Can;
use tokio::fs::File;

#[allow(dead_code)]
impl<T: Can> UdsClient<'_, T> {
    /// Service ID: 0x36 - Transfer Data
    /// Description:
    ///     The function will request a data transfer from ECU.
    ///     The data will store in the <file> as raw binary
    pub async fn get_ecu_log(&mut self, mut file: File) -> Result<(), DiagError> {
        todo!()
    }
}
