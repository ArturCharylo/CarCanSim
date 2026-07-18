pub mod hardware;
pub mod simulator;

pub trait ObdInterface: Send + Sync {
    fn read_engine_rpm(&self) -> Result<u32, String>;
    fn read_vehicle_speed(&self) -> Result<u8, String>;
}
