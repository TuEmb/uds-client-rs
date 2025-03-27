use uds_client::{CanSocketTx, UdsClient};
use automotive_diag::uds::UdsCommand;


pub trait UdsClientService {
    async fn run_service(&mut self, sid: UdsCommand);
}

impl<'a, T: CanSocketTx> UdsClientService for UdsClient<'a, T> {
    async fn run_service(&mut self, sid: UdsCommand) {
        match sid {
            UdsCommand::DiagnosticSessionControl => uds_session_control(self).await,
            UdsCommand::ECUReset => uds_reset_service(self).await,
            UdsCommand::SecurityAccess => uds_security_access(self).await,
            UdsCommand::CommunicationControl => uds_communication_control(self).await,
            UdsCommand::TesterPresent => todo!(),
            UdsCommand::Authentication => todo!(),
            UdsCommand::SecuredDataTransmission => todo!(),
            UdsCommand::ControlDTCSetting => todo!(),
            UdsCommand::ResponseOnEvent => todo!(),
            UdsCommand::LinkControl => todo!(),
            UdsCommand::ReadDataByIdentifier => todo!(),
            UdsCommand::ReadMemoryByAddress => todo!(),
            UdsCommand::ReadScalingDataByIdentifier => todo!(),
            UdsCommand::ReadDataByPeriodicIdentifier => todo!(),
            UdsCommand::DynamicallyDefineDataIdentifier => todo!(),
            UdsCommand::WriteDataByIdentifier => todo!(),
            UdsCommand::WriteMemoryByAddress => todo!(),
            UdsCommand::ClearDiagnosticInformation => todo!(),
            UdsCommand::ReadDTCInformation => todo!(),
            UdsCommand::InputOutputControlByIdentifier => todo!(),
            UdsCommand::RoutineControl => todo!(),
            UdsCommand::RequestDownload => todo!(),
            UdsCommand::RequestUpload => todo!(),
            UdsCommand::TransferData => todo!(),
            UdsCommand::RequestTransferExit => todo!(),
            UdsCommand::RequestFileTransfer => todo!(),
        }
    }
}

async fn uds_session_control<'a, T: CanSocketTx>(_client: &mut UdsClient<'a, T>) {
    println!("run uds_session_control");
}

async fn uds_reset_service<'a, T: CanSocketTx>(_client: &mut UdsClient<'a, T>) {
    println!("run uds_reset_service");
}

async fn uds_security_access<'a, T: CanSocketTx>(_client: &mut UdsClient<'a, T>) {
    println!("run uds_security_access");
}

async fn uds_communication_control<'a, T: CanSocketTx>(_client: &mut UdsClient<'a, T>) {
    println!("run uds_communication_control");
}

