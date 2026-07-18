use crate::obd::ObdInterface;
use rand::RngExt;

pub struct Simulator;

impl ObdInterface for Simulator {
    fn read_engine_rpm(&self) -> Result<u32, String> {
        let mut rng = rand::rng();
        let rpm: u32 = rng.random_range(800..8500);
        Ok(rpm)
    }

    fn read_vehicle_speed(&self) -> Result<u8, String> {
        let mut rng = rand::rng();
        let speed: u8 = rng.random_range(0..100);
        Ok(speed)
    }
}
