use crate::{
    socket_can::CanSocketTx,
    uds_client::{DiagError, UdsClient},
};
use tokio::fs::File;

#[allow(dead_code)]
impl<T: CanSocketTx> UdsClient<'_, T> {
    /// Service ID: 0x36 - Transfer Data
    /// Description:
    ///     The function will request a data transfer from ECU.
    ///     The data will store in the <file> as raw binary
    pub async fn get_ecu_log(&mut self, mut _file: File) -> Result<(), DiagError> {
        todo!()
    }
}
