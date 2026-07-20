pub mod hardware;
pub mod simulator;

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

pub trait ObdInterface: Send + Sync {
    fn read_engine_rpm(&self) -> Result<u32, ObdError>;
    fn read_vehicle_speed(&self) -> Result<u8, ObdError>;
    fn read_oil_temp(&self) -> Result<f32, ObdError>;
    fn read_error_code(&self) -> Result<u8, ObdError>;
}
