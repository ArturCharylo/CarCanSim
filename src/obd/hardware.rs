use crate::obd::{ObdInterface, ObdError};

pub struct HardwareAdapter;

impl ObdInterface for HardwareAdapter {
    fn read_engine_rpm(&self) -> Result<u32, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }

    fn read_oil_temp(&self) -> Result<f32, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }

    fn read_error_code(&self) -> Result<u8, ObdError> {
        Err(ObdError::NotImplemented("Hardware integration not implemented yet".to_string()))
    }
}
