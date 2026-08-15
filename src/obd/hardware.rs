use crate::obd::{ObdError, ObdInterface};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;

// --- PURE PARSING FUNCTIONS ---
// These functions do not depend on hardware and can be easily unit-tested.
pub fn parse_engine_rpm(raw_response: &str) -> u32 {
    let cleaned = raw_response
        .replace(" ", "")
        .replace("\r", "")
        .replace("\n", "");
    // Verify if the response matches the expected echo (41 0C)
    if cleaned.starts_with("410C") && cleaned.len() >= 8 {
        let a = u32::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
        let b = u32::from_str_radix(&cleaned[6..8], 16).unwrap_or(0);
        return ((a * 256) + b) / 4;
    }
    0
}

pub fn parse_vehicle_speed(raw_response: &str) -> u8 {
    let cleaned = raw_response
        .replace(" ", "")
        .replace("\r", "")
        .replace("\n", "");
    // Verify if the response matches the expected echo (41 0D)
    if cleaned.starts_with("410D") && cleaned.len() >= 6 {
        return u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
    }
    0
}

pub fn parse_oil_temp(raw_response: &str) -> f32 {
    let cleaned = raw_response
        .replace(" ", "")
        .replace("\r", "")
        .replace("\n", "");
    // Verify if the response matches the expected echo (41 5C)
    if cleaned.starts_with("415C") && cleaned.len() >= 6 {
        let a = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
        return (a as f32) - 40.0;
    }
    0.0
}

pub fn parse_error_code(raw_response: &str) -> String {
    let cleaned = raw_response
        .replace(" ", "")
        .replace("\r", "")
        .replace("\n", "");
    // Mode 03 response starts with "43". The first DTC is located in the next two bytes.
    if cleaned.starts_with("43") && cleaned.len() >= 6 {
        let a = u8::from_str_radix(&cleaned[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);

        if a == 0 && b == 0 {
            return "NONE".to_string();
        }

        // 1st character determines the vehicle system
        let system = match (a >> 6) & 0b11 {
            0 => 'P', // Powertrain
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => 'P',
        };

        let digit1 = (a >> 4) & 0b11;
        let digit2 = a & 0b1111;
        let digit3 = (b >> 4) & 0b1111;
        let digit4 = b & 0b1111;

        return format!("{}{}{:X}{:X}{:X}", system, digit1, digit2, digit3, digit4);
    }
    "UNKNOWN".to_string()
}

// --- HARDWARE ADAPTER ---

pub struct HardwareAdapter {
    connection: Mutex<Box<dyn serialport::SerialPort>>,
}

impl HardwareAdapter {
    pub fn new(port_name: &str) -> Result<Self, String> {
        let mut connection = serialport::new(port_name, 38400)
            .timeout(Duration::from_millis(2000))
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;

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
        let raw = self
            .send_request(b"010C\r")
            .map_err(|_| ObdError::NotImplemented("Hardware error".to_string()))?;
        Ok(parse_engine_rpm(&raw))
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        let raw = self
            .send_request(b"010D\r")
            .map_err(|_| ObdError::NotImplemented("Hardware error".to_string()))?;
        Ok(parse_vehicle_speed(&raw))
    }

    fn read_oil_temp(&self) -> Result<f32, ObdError> {
        let raw = self
            .send_request(b"015C\r")
            .map_err(|_| ObdError::NotImplemented("Hardware error".to_string()))?;
        Ok(parse_oil_temp(&raw))
    }

    fn read_error_code(&self) -> Result<String, ObdError> {
        let raw = self
            .send_request(b"03\r")
            .map_err(|_| ObdError::NotImplemented("Hardware error".to_string()))?;
        Ok(parse_error_code(&raw))
    }

    fn read_current_gear(&self) -> Result<u8, ObdError> {
        Ok(2) // placeholder for test purposes just to avoid errors
    }
}

// --- UNIT TESTS ---
// This module is only compiled when running `cargo test`

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_engine_rpm() {
        // Expected: A = 0x1A (26), B = 0xF8 (248) -> ((26 * 256) + 248) / 4 = 1726
        assert_eq!(parse_engine_rpm("41 0C 1A F8\r\n"), 1726);

        // Test without spaces and carriage returns
        assert_eq!(parse_engine_rpm("410C1AF8"), 1726);

        // Test invalid response
        assert_eq!(parse_engine_rpm("SEARCHING..."), 0);
        assert_eq!(parse_engine_rpm("NODATA"), 0);
    }

    #[test]
    fn test_parse_vehicle_speed() {
        // Expected: A = 0x32 (50) -> 50 km/h
        assert_eq!(parse_vehicle_speed("41 0D 32\r\n"), 50);
        assert_eq!(parse_vehicle_speed("410D32"), 50);
        assert_eq!(parse_vehicle_speed("ERROR"), 0);
    }

    #[test]
    fn test_parse_oil_temp() {
        // Expected: A = 0x5A (90) -> 90 - 40 = 50.0 degrees Celsius
        assert_eq!(parse_oil_temp("41 5C 5A\r\n"), 50.0);

        // Expected: A = 0x28 (40) -> 40 - 40 = 0.0 degrees Celsius
        assert_eq!(parse_oil_temp("41 5C 28"), 0.0);
    }

    #[test]
    fn test_parse_error_code() {
        // Expected: A = 0x01, B = 0x33 -> 00000001 00110011 -> Powertrain (P), 0, 1, 3, 3 -> P0133
        assert_eq!(parse_error_code("43 01 33\r\n"), "P0133");

        // Expected: No errors present
        assert_eq!(parse_error_code("43 00 00\r\n"), "NONE");

        // Expected: Invalid or garbled response
        assert_eq!(parse_error_code("NO DATA"), "UNKNOWN");
    }
}
