#[cfg(target_os = "linux")]
use embedded_can::Frame;
use log::{error, info};
use socket_can::{CanSocketRx, UdsSocketRx};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::mpsc;
use uds::uds_client_task;
use uds_client::ResponseSlot;
use ui::UiEventTx;

mod socket_can;
mod uds;
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
    let (tx_socket, rx_socket) = socket_can::UdsSocket::new("can0").split();
    #[cfg(target_os = "windows")]
    let (tx_socket, mut rx_socket) = socket_can::UdsSocket::new().split();
    let (ui_tx, uds_rx) = mpsc::channel::<UiEventTx>(10);

    let ui = MainWindow::new().unwrap();
    ui.on_reset(move |chip| {
        let reset_tx = ui_tx.clone();
        tokio::spawn(async move {
            let _ = reset_tx.send(chip.try_into().unwrap()).await;
        });
    });

    // Create UDS client task
    uds_client_task(tx_socket, uds_rx).await.ok();
    response_task(rx_socket).await.ok();

    // start UI
    let _ = ui.run();
}

/// The response task: handle Rx UDS socket and update to RESPONSE_SLOT
pub async fn response_task(mut rx_socket: UdsSocketRx) -> Result<(), ()> {
    tokio::spawn(async move {
        loop {
            if let Ok(frame) = rx_socket.receive_with_timeout(Duration::from_millis(10)) {
                info!("Received frame: {:?}", frame);
                if let Err(e) = RESPONSE_SLOT.update_response(frame.data().to_vec()).await {
                    error!("UDS: Failed to update response from UDS server: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    Ok(())
}
