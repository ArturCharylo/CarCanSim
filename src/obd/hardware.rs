use crate::obd::ObdInterface;

pub struct HardwareAdapter;

impl ObdInterface for HardwareAdapter {
    fn read_engine_rpm(&self) -> Result<u32, String> {
        Err("Hardware integration not implemented yet".to_string())
    }

    fn read_vehicle_speed(&self) -> Result<u8, String> {
        Err("Hardware integration not implemented yet".to_string())
    }
}
