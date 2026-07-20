use crate::obd::ObdInterface;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObdError{
    #[error("Hardware disconnected")]
    Disconnected,
    #[error("Failed to parse metrics")]
    ParseError,
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

pub struct HardwareAdapter;

impl ObdInterface for HardwareAdapter {
    fn read_engine_rpm(&self) -> Result<u32, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }
}
