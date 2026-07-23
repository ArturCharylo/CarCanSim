use crate::obd::{ObdInterface, ObdError};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;

pub struct HardwareAdapter {
    connection: Mutex<Box<dyn serialport::SerialPort>>,
}

impl HardwareAdapter {
    // Initialize the connection with the given Bluetooth COM/rfcomm port
    pub fn new(port_name: &str) -> Result<Self, String> {
        let mut connection = serialport::new(port_name, 38400)
            .timeout(Duration::from_millis(2000))
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;

        // Initialize ELM327 adapter: reset device and disable terminal echo
        connection.write_all(b"ATZ\r").map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(500));
        
        connection.write_all(b"ATE0\r").map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(100));

        Ok(HardwareAdapter {
            connection: Mutex::new(connection),
        })
    }

    fn send_request(&self, command: &[u8]) -> Result<String, String> {
        let mut conn = self.connection.lock().map_err(|_| "Failed to lock mutex")?;

        conn.write_all(command).map_err(|e| e.to_string())?;

        let mut buffer = [0; 128];
        let bytes_read = conn.read(&mut buffer).map_err(|e| e.to_string())?;

        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
    }
}

impl ObdInterface for HardwareAdapter {
    fn read_engine_rpm(&self) -> Result<u32, ObdError> {
        let raw_response = self.send_request(b"010C\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
       let cleaned = raw_response.replace(" ", "").replace("\r", "").replace("\n", "");
        
        // Verify if the response matches the expected echo (41 0C)
        if cleaned.starts_with("410C") && cleaned.len() >= 8 {
            // Extract byte A and byte B as hex strings
            let a_hex = &cleaned[4..6];
            let b_hex = &cleaned[6..8];
            
            // Parse hex strings into integers
            let a = u32::from_str_radix(a_hex, 16).unwrap_or(0);
            let b = u32::from_str_radix(b_hex, 16).unwrap_or(0);
            
            let rpm = ((a * 256) + b) / 4;
            return Ok(rpm);
        }
        
        Ok(0)
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        let raw_response = self.send_request(b"010D\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        let cleaned = raw_response.replace(" ", "").replace("\r", "").replace("\n", "");

        if cleaned.starts_with("410D") && cleaned.len() >= 6{
            let velocity = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
            return Ok(velocity);
        }

        Ok(0)
    }

    fn read_oil_temp(&self) -> Result<f32, ObdError> {
        let raw_response = self.send_request(b"015C\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        let cleaned = raw_response.replace(" ", "").replace("\r", "").replace("\n", "");

        if cleaned.starts_with("415C") && cleaned.len() >= 6{
            let a = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
            let oil_temp = (a as f32) - 40.0;
            return Ok(oil_temp);
        }
        Ok(0.0)
    }

    fn read_error_code(&self) -> Result<String, ObdError> {
        let raw_response = self.send_request(b"03\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
         let cleaned = raw_response.replace(" ", "").replace("\r", "").replace("\n", "");
        
        // Mode 03 response starts with "43". The first DTC is located in the next two bytes.
        if cleaned.starts_with("43") && cleaned.len() >= 6 {
            let a_hex = &cleaned[2..4];
            let b_hex = &cleaned[4..6];
            
            let a = u8::from_str_radix(a_hex, 16).unwrap_or(0);
            let b = u8::from_str_radix(b_hex, 16).unwrap_or(0);
            
            if a == 0 && b == 0 {
                return Ok("NONE".to_string());
            }

            // 1st character determines the vehicle system
            let system = match (a >> 6) & 0b11 {
                0 => 'P', // Powertrain
                1 => 'C', // Chassis
                2 => 'B', // Body
                3 => 'U', // Network
                _ => 'P',
            };
            
            // Extract the remaining hex digits
            let digit1 = (a >> 4) & 0b11;
            let digit2 = a & 0b1111;
            let digit3 = (b >> 4) & 0b1111;
            let digit4 = b & 0b1111;
            
            let dtc = format!("{}{}{:X}{:X}{:X}", system, digit1, digit2, digit3, digit4);
            return Ok(dtc);
        }
        
        Ok("UNKNOWN".to_string())
    
    }
}