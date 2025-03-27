impl TryFrom<i32> for UiEventTx {
    type Error = ();
    fn try_from(ui_type: i32) -> Result<Self, Self::Error> {
        match ui_type as u8 {
            0x01 => Ok(UiEventTx::EcuReset),
            0x02 => Ok(UiEventTx::SecurityAccess),
            0x03 => Ok(UiEventTx::CommunicationControl),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum UiEventTx {
    EcuReset,
    SecurityAccess,
    CommunicationControl
}
