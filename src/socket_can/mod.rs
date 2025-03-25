#[cfg(target_os = "windows")]
use embedded_can::ExtendedId;
use embedded_can::{Frame, nb::Can};
use embedded_io_async::ErrorType;
#[cfg(target_os = "windows")]
use peak_can::{
    bus::UsbBus,
    df::ReceiveStatus,
    error::CanError,
    socket::usb::UsbCanSocket,
    socket::{Baudrate, RecvCan, SendCan},
    socket::{CanFrame, MessageType},
};
#[cfg(target_os = "linux")]
use socketcan::{CanFrame, CanSocket, Socket};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct WrappedCanFrame(pub CanFrame);
#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WrappedPcanError(pub CanError);

pub trait CanSocketTx {
    /// Associated frame type.
    type Frame: Frame;

    /// Associated error type.
    type Error: embedded_can::Error;

    // The transmit function
    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error>;
}

#[allow(dead_code)]
pub trait CanSocketRx {
    /// Associated frame type.
    type Frame: Frame;

    /// Associated error type.
    type Error: embedded_can::Error;

    // The receive function
    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error>;
}

pub struct UdsSocket {
    #[cfg(target_os = "linux")]
    can_socket: CanSocket,
    #[cfg(target_os = "windows")]
    can_socket: UsbCanSocket,
}

pub struct UdsSocketTx {
    #[cfg(target_os = "linux")]
    tx: Arc<Mutex<CanSocket>>,
    #[cfg(target_os = "windows")]
    tx: Arc<Mutex<UsbCanSocket>>,
}

pub struct UdsSocketRx {
    #[cfg(target_os = "linux")]
    rx: Arc<Mutex<CanSocket>>,
    #[cfg(target_os = "windows")]
    rx: Arc<Mutex<UsbCanSocket>>,
}

impl UdsSocket {
    #[cfg(target_os = "linux")]
    pub fn new(socket: &str) -> Self {
        use socketcan::{CanFilter, SocketOptions};

        let can_socket = CanSocket::open(socket).unwrap();
        let filter = CanFilter::new(0x7F0, 0x1FFFFFFF);
        let _ = can_socket.set_filters(&[filter]);
        Self { can_socket }
    }

    #[cfg(target_os = "windows")]
    pub fn new() -> Self {
        let can_socket = match UsbCanSocket::open(UsbBus::USB1, Baudrate::Baud500K) {
            Ok(socket) => socket,
            Err(e) => {
                log::warn!("The PCAN initialize failed {:?}, just open", e);
                UsbCanSocket::open_with_usb_bus(UsbBus::USB1)
            }
        };
        Self { can_socket }
    }

    pub fn split(self) -> (UdsSocketTx, UdsSocketRx) {
        let shared_socket = Arc::new(Mutex::new(self.can_socket));
        let rx_socket = UdsSocketRx {
            rx: shared_socket.clone(),
        };
        let tx_socket = UdsSocketTx {
            tx: shared_socket.clone(),
        };
        (tx_socket, rx_socket)
    }
}

// Specify the error type for `embedded_io_async`
impl ErrorType for UdsSocket {
    type Error = embedded_io_async::ErrorKind;
}

#[cfg(target_os = "linux")]
impl Can for UdsSocket {
    type Frame = CanFrame;

    type Error = socketcan::Error;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
        self.can_socket.transmit(frame)
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.can_socket.receive()
    }
}

#[cfg(target_os = "linux")]
impl CanSocketTx for UdsSocketTx {
    type Frame = CanFrame;
    type Error = socketcan::Error;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
        self.tx.lock().unwrap().transmit(frame)
    }
}

#[cfg(target_os = "linux")]
impl CanSocketRx for UdsSocketRx {
    type Frame = CanFrame;
    type Error = socketcan::Error;

    fn receive(&mut self) -> nb::Result<CanFrame, socketcan::Error> {
        self.rx.lock().unwrap().receive()
    }
}

#[cfg(target_os = "linux")]
impl UdsSocketRx {
    pub fn receive_with_timeout(&mut self, timeout: Duration) -> socketcan::IoResult<CanFrame> {
        self.rx.lock().unwrap().read_frame_timeout(timeout)
    }
}

#[cfg(target_os = "windows")]
impl embedded_can::Error for WrappedPcanError {
    fn kind(&self) -> embedded_can::ErrorKind {
        match self.0 {
            CanError::Overrun => embedded_can::ErrorKind::Overrun,
            _ => embedded_can::ErrorKind::Other,
        }
    }
}

#[cfg(target_os = "windows")]
impl embedded_can::Frame for WrappedCanFrame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        let can_id: embedded_can::Id = id.into();
        let raw_id = match can_id {
            embedded_can::Id::Standard(standard_id) => standard_id.as_raw() as u32,
            embedded_can::Id::Extended(extended_id) => extended_id.as_raw(),
        };
        match CanFrame::new(raw_id, MessageType::Extended, data) {
            Ok(frame) => Some(WrappedCanFrame(frame)),
            Err(_) => None,
        }
    }

    fn id(&self) -> embedded_can::Id {
        embedded_can::Id::Extended(ExtendedId::new(self.0.can_id()).unwrap())
    }

    fn data(&self) -> &[u8] {
        self.0.data()
    }

    fn new_remote(id: impl Into<embedded_can::Id>, dlc: usize) -> Option<Self> {
        let _ = dlc;
        let _ = id;
        todo!()
    }

    fn is_extended(&self) -> bool {
        self.0.is_extended_frame()
    }

    fn is_remote_frame(&self) -> bool {
        todo!()
    }

    fn dlc(&self) -> usize {
        self.0.dlc() as usize
    }

    fn is_standard(&self) -> bool {
        !self.0.is_extended_frame()
    }

    fn is_data_frame(&self) -> bool {
        todo!()
    }
}

#[cfg(target_os = "windows")]
impl Can for UdsSocket {
    type Frame = WrappedCanFrame;

    type Error = WrappedPcanError;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
        match self.can_socket.send(frame.0) {
            Ok(_) => Ok(Some(Self::Frame::default())),
            Err(e) => Err(nb::Error::Other(WrappedPcanError(e))),
        }
    }

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        match self.can_socket.recv() {
            Ok(f) => Ok(WrappedCanFrame(f.0)),
            Err(e) => Err(nb::Error::Other(WrappedPcanError(e))),
        }
    }
}

#[cfg(target_os = "windows")]
impl CanSocketTx for UdsSocketTx {
    type Frame = WrappedCanFrame;

    type Error = WrappedPcanError;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
        match self.tx.lock().unwrap().send(frame.0) {
            Ok(_) => Ok(Some(Self::Frame::default())),
            Err(e) => Err(nb::Error::Other(WrappedPcanError(e))),
        }
    }
}

#[cfg(target_os = "windows")]
impl CanSocketRx for UdsSocketRx {
    type Frame = WrappedCanFrame;

    type Error = WrappedPcanError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        match self.rx.lock().unwrap().recv() {
            Ok(f) => Ok(WrappedCanFrame(f.0)),
            Err(e) => Err(nb::Error::Other(WrappedPcanError(e))),
        }
    }
}

#[cfg(target_os = "windows")]
impl UdsSocketRx {
    pub fn receive_with_timeout(&mut self, timeout: Duration) -> Result<CanFrame, CanError> {
        let start = chrono::Local::now();
        while !self.rx.lock().unwrap().is_receiving()? {
            if chrono::Local::now() > start + timeout {
                return Err(CanError::Unknown);
            }
        }

        self.rx.lock().unwrap().recv_frame()
    }
}
