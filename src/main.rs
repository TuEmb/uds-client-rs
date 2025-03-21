#[cfg(target_os = "linux")]
use embedded_can::Frame;
use log::{error, info, warn};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::mpsc;
use uds_client::{ResetType, ResponseSlot};
use ui::UiEventTx;

mod socket_can;
mod uds_client;
mod ui;

slint::include_modules!();
pub static RESPONSE_SLOT: LazyLock<Arc<ResponseSlot>> =
    LazyLock::new(|| Arc::new(ResponseSlot::new()));

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();
    #[cfg(target_os = "linux")]
    let (mut tx_socket, mut rx_socket) = socket_can::UdsSocket::new("can0").split();
    #[cfg(target_os = "windows")]
    let (mut tx_socket, mut rx_socket) = socket_can::UdsSocket::new().split();
    let (ui_tx, mut uds_rx) = mpsc::channel::<UiEventTx>(10);

    let ui = MainWindow::new().unwrap();
    ui.on_reset(move |chip| {
        let reset_tx = ui_tx.clone();
        tokio::spawn(async move {
            let _ = reset_tx.send(chip.try_into().unwrap()).await;
        });
    });
    // first thread for UDS service
    tokio::spawn(async move {
        let mut uds_client = uds_client::UdsClient::new(&mut tx_socket, 0x784, &RESPONSE_SLOT);
        let file = tokio::fs::File::create("./log.bin").await.unwrap();
        if let Err(e) = uds_client.get_ecu_log(file).await {
            warn!("Failed to get ECU log: {e:?}");
        }
        info!("Got log from ECU successfully");
        while let Some(event) = uds_rx.recv().await {
            match event {
                UiEventTx::Reset(reset_type) => match reset_type {
                    ResetType::RealTime => uds_client.uds_reset_118().await.unwrap(),
                    ResetType::Telematic => uds_client.uds_reset_148().await.unwrap(),
                    ResetType::Imx => uds_client.uds_reset_imx().await.unwrap(),
                    ResetType::Esp32Wifi => uds_client.uds_reset_esp32_wifi().await.unwrap(),
                    ResetType::Esp32Ble => uds_client.uds_reset_esp32_ble().await.unwrap(),
                    ResetType::Lte => uds_client.uds_reset_lte().await.unwrap(),
                    ResetType::Lizard => uds_client.uds_reset_lizard().await.unwrap(),
                    ResetType::Cendric => uds_client.uds_reset_cendric().await.unwrap(),
                },
                UiEventTx::RealTime(real_time_type) => match real_time_type {
                    uds_client::RealTimeType::SlowRate => {
                        uds_client.uds_real_time_data_slow().await.unwrap()
                    }
                    uds_client::RealTimeType::MediumRate => {
                        uds_client.uds_real_time_data_medium().await.unwrap()
                    }
                    uds_client::RealTimeType::FastRate => {
                        uds_client.uds_real_time_data_fast().await.unwrap()
                    }
                    uds_client::RealTimeType::Stop => {
                        uds_client.uds_real_time_data_stop().await.unwrap()
                    }
                },
            }
        }
    });
    let response_slot = RESPONSE_SLOT.clone();

    // second thread for response
    tokio::spawn(async move {
        loop {
            if let Ok(frame) = rx_socket.receive_with_timeout(Duration::from_millis(10)) {
                info!("Received frame: {:?}", frame);
                if let Err(e) = response_slot.update_response(frame.data().to_vec()).await {
                    error!("UDS: Failed to update response from UDS server: {:?}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let _ = ui.run();
}
