use std::sync::Arc;

use log::info;
use tokio::sync::{Mutex, mpsc::Receiver};
use uds_client_rs::{RealTimeType, ResetType, UdsClient, UdsSocketTx};

use crate::{ui::UiEventTx, RESPONSE_SLOT};

/// The UDS client task: receive and process the event from UI
pub async fn uds_client_task(
    tx_socket: UdsSocketTx,
    mut uds_rx: Receiver<UiEventTx>,
) -> Result<(), ()> {
    tokio::spawn(async move {
        let uds_client = Arc::new(Mutex::new(UdsClient::new(tx_socket, 0x784, &RESPONSE_SLOT)));

        info!("Got log from ECU successfully");
        while let Some(event) = uds_rx.recv().await {
            let uds_client_clone = Arc::clone(&uds_client);
            tokio::spawn(async move {
                info!("Received event from UI: {:?}", event);
                match event {
                    UiEventTx::Reset(reset_type) => match reset_type {
                        ResetType::RealTime => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_118()
                            .await
                            .unwrap(),
                        ResetType::Telematic => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_148()
                            .await
                            .unwrap(),
                        ResetType::Imx => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_imx()
                            .await
                            .unwrap(),
                        ResetType::Esp32Wifi => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_esp32_wifi()
                            .await
                            .unwrap(),
                        ResetType::Esp32Ble => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_esp32_ble()
                            .await
                            .unwrap(),
                        ResetType::Lte => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_lte()
                            .await
                            .unwrap(),
                        ResetType::Lizard => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_lizard()
                            .await
                            .unwrap(),
                        ResetType::Cendric => uds_client_clone
                            .lock()
                            .await
                            .uds_reset_cendric()
                            .await
                            .unwrap(),
                    },
                    UiEventTx::RealTime(real_time_type) => match real_time_type {
                        RealTimeType::SlowRate => uds_client_clone
                            .lock()
                            .await
                            .uds_real_time_data_slow()
                            .await
                            .unwrap(),
                        RealTimeType::MediumRate => uds_client_clone
                            .lock()
                            .await
                            .uds_real_time_data_medium()
                            .await
                            .unwrap(),
                        RealTimeType::FastRate => uds_client_clone
                            .lock()
                            .await
                            .uds_real_time_data_fast()
                            .await
                            .unwrap(),
                        RealTimeType::Stop => uds_client_clone
                            .lock()
                            .await
                            .uds_real_time_data_stop()
                            .await
                            .unwrap(),
                    },
                }
                info!("UDS: process event finished OK");
            });
        }
    });

    Ok(())
}
