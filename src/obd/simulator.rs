use crate::obd::{ObdError, ObdInterface};
use rand::RngExt;
use std::sync::Mutex;
use std::time::Instant;

// Internal state to keep track of simulated vehicle physics
struct VehicleState {
    speed: f32,
    accelerating: bool,
    last_update: Instant,
    oil_temp: f32,
    current_rpm: u32,
    current_gear: u8,
}

pub struct Simulator {
    // Mutex allows interior mutability for &self methods
    state: Mutex<VehicleState>,
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VehicleState {
                speed: 0.0,
                accelerating: true,
                last_update: Instant::now(),
                oil_temp: 20.0,
                current_rpm: 800,
                current_gear: 1,
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

        // Simulate acceleration and deceleration logic with aerodynamic drag
        if state.accelerating {
            // Acceleration decreases at higher speeds (simulating air resistance and engine load)
            let acceleration_factor = (130.0 - state.speed).max(10.0) / 10.0;
            state.speed += rng.random_range(2.0..6.0) * acceleration_factor * delta_time;
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

        // Automatic transmission shift logic
        state.current_gear = match state.speed as u8 {
            0..=15 => 1,
            16..=30 => 2,
            31..=50 => 3,
            51..=70 => 4,
            71..=90 => 5,
            91..=110 => 6,
            _ => 7,
        };

        state.speed as u8
    }
}

impl ObdInterface for Simulator {
    fn read_engine_rpm(&self) -> Result<u32, ObdError> {
        let speed = self.update_and_get_speed();
        let gear = {
            let state = self.state.lock().unwrap();
            state.current_gear
        };

        // Calculate RPM considering the current gear and speed
        // This simulates the RPM drop after shifting gears in an automatic transmission
        let base_rpm = match gear {
            1 => 800 + (speed as u32 * 100),
            2 => 1200 + ((speed.saturating_sub(15)) as u32 * 80),
            3 => 1400 + ((speed.saturating_sub(30)) as u32 * 60),
            4 => 1500 + ((speed.saturating_sub(50)) as u32 * 50),
            5 => 1600 + ((speed.saturating_sub(70)) as u32 * 45),
            6 => 1700 + ((speed.saturating_sub(90)) as u32 * 40),
            _ => 1800 + ((speed.saturating_sub(110)) as u32 * 35),
        };

        // minor mechanical jitter for realism
        let mut rng = rand::rng();
        let jitter: i32 = rng.random_range(-30..30);

        let final_rpm = (base_rpm as i32 + jitter).max(800) as u32;
        if let Ok(mut state) = self.state.lock() {
            state.current_rpm = final_rpm;
        }

        Ok(final_rpm)
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        let speed = self.update_and_get_speed();
        Ok(speed)
    }

    fn read_oil_temp(&self) -> Result<f32, ObdError> {
        let mut state = self.state.lock().unwrap();

        let target_temp = 85.0 + (state.current_rpm as f32 / 350.0);

        let diff = target_temp - state.oil_temp;
        state.oil_temp += diff * 0.02;

        let mut rng = rand::rng();
        let fluctuation: f32 = rng.random_range(-0.3..0.3);

        Ok(state.oil_temp + fluctuation)
    }

    fn read_error_code(&self) -> Result<String, ObdError> {
        let mut rng = rand::rng();
        // 1% chance to simulate an active DTC (Diagnostic Trouble Code) for monitoring tests
        if rng.random_range(0..100) < 1 {
            Ok("P012C".to_string())
        } else {
            Ok("NONE".to_string())
        }
    }

    fn read_current_gear(&self) -> Result<u8, ObdError> {
        let gear = {
            let state = self.state.lock().unwrap();
            state.current_gear
        };
        Ok(gear)
    }
}
