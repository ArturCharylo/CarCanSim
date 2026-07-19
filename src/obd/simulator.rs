use crate::obd::ObdInterface;
use rand::RngExt; 
use std::sync::Mutex;
use std::time::Instant;

// Internal state to keep track of simulated vehicle physics
struct VehicleState {
    speed: f32,
    accelerating: bool,
    last_update: Instant,
}

pub struct Simulator {
    // Mutex allows interior mutability for &self methods
    state: Mutex<VehicleState>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VehicleState {
                speed: 0.0,
                accelerating: true,
                last_update: Instant::now(),
            }),
        }
    }

    // Helper function to update speed smoothly based on elapsed time
    fn update_and_get_speed(&self) -> u8 {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let delta_time = now.duration_since(state.last_update).as_secs_f32();
        state.last_update = now;

        let mut rng = rand::rng();

        // Simulate acceleration and deceleration logic
        if state.accelerating {
            state.speed += rng.random_range(5.0..15.0) * delta_time;
            if state.speed >= 120.0 {
                state.accelerating = false; 
            }
        } else {
            state.speed -= rng.random_range(5.0..15.0) * delta_time;
            if state.speed <= 0.0 {
                state.speed = 0.0;
                state.accelerating = true; 
            }
        }

        state.speed as u8
    }
}

impl ObdInterface for Simulator {
    fn read_engine_rpm(&self) -> Result<u32, String> {
        let speed = self.update_and_get_speed();

        // Calculate RPM based on current speed and simulated gear ratios
        let base_rpm = match speed {
            0 => 800,
            1..=25 => 800 + (speed as u32 * 80),
            26..=50 => 1200 + ((speed - 25) as u32 * 60),
            51..=80 => 1500 + ((speed - 50) as u32 * 50),
            81..=110 => 1800 + ((speed - 80) as u32 * 40),
            _ => 2000 + ((speed - 110) as u32 * 35),
        };

        // Add minor mechanical jitter for realism
        let mut rng = rand::rng();
        let jitter: i32 = rng.random_range(-30..30);
        
        let final_rpm = (base_rpm as i32 + jitter).max(800) as u32;

        Ok(final_rpm)
    }

    fn read_vehicle_speed(&self) -> Result<u8, String> {
        let speed = self.update_and_get_speed();
        Ok(speed)
    }
}