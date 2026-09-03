use crate::obd::{ObdError, ObdInterface};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;

// --- PURE PARSING FUNCTIONS ---
// These functions do not depend on hardware and can be easily unit-tested.
pub fn parse_engine_rpm(raw_response: &str) -> u32 {
    let cleaned = raw_response
        .replace(' ', "")
        .replace('\r', "")
        .replace('\n', "");

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
        .replace(' ', "")
        .replace('\r', "")
        .replace('\n', "");

    // Verify if the response matches the expected echo (41 0D)
    if cleaned.starts_with("410D") && cleaned.len() >= 6 {
        return u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
    }
    0
}

pub fn parse_oil_temp(raw_response: &str) -> f32 {
    let cleaned = raw_response
        .replace(' ', "")
        .replace('\r', "")
        .replace('\n', "");

    // Verify if the response matches the expected echo (41 5C)
    if cleaned.starts_with("415C") && cleaned.len() >= 6 {
        let a = u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
        return (a as f32) - 40.0;
    }
    0.0
}

pub fn parse_error_code(raw_response: &str) -> String {
    let cleaned = raw_response
        .replace(' ', "")
        .replace('\r', "")
        .replace('\n', "");

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

// Parses the standard OBD-II PID 01 A4 for current gear.
// Note: Very few vehicles actually support this PID in Mode 01.
pub fn parse_current_gear_pid(raw_response: &str) -> u8 {
    let cleaned = raw_response
        .replace(' ', "")
        .replace('\r', "")
        .replace('\n', "");

    // Verify if the response matches the expected echo (41 A4)
    if cleaned.starts_with("41A4") && cleaned.len() >= 8 {
        return u8::from_str_radix(&cleaned[4..6], 16).unwrap_or(0);
    }
    0
}

// Calculates the current gear based on the ratio between RPM and Vehicle Speed.
pub fn calculate_gear_from_ratio(rpm: u32, speed: u8) -> u8 {
    if speed == 0 || rpm < 800 {
        return 0; // Neutral or vehicle is stopped
    }

    let ratio = rpm as f32 / speed as f32;

    // Example thresholds for manual/calibrated ratios
    if ratio > 120.0 {
        1
    } else if ratio > 75.0 {
        2
    } else if ratio > 50.0 {
        3
    } else if ratio > 38.0 {
        4
    } else if ratio > 28.0 {
        5
    } else if ratio > 15.0 {
        6
    } else {
        0
    }
}

// --- HARDWARE ADAPTER ---

pub struct HardwareAdapter {
    connection: Mutex<Box<dyn serialport::SerialPort>>,
}

impl HardwareAdapter {
    pub fn new(port_name: &str) -> Result<Self, String> {
        let connection = serialport::new(port_name, 38400)
            .timeout(Duration::from_millis(1500))
            .open()
            .map_err(|e| format!("Failed to open port {}: {}", port_name, e))?;

        let adapter = HardwareAdapter {
            connection: Mutex::new(connection),
        };

        // Reset ELM327
        let _ = adapter.send_request(b"ATZ\r");
        std::thread::sleep(Duration::from_millis(500));

        // Disable echo
        let _ = adapter.send_request(b"ATE0\r");

        // Set protocol to automatic
        let _ = adapter.send_request(b"ATSP0\r");

        // Test vehicle communication (PID 0100 queries supported PIDs 01-20)
        let test_response = adapter
            .send_request(b"0100\r")
            .map_err(|e| format!("Failed to send handshake PID: {}", e))?;

        // Check whether the ECU responded or connection failed
        if test_response.contains("UNABLE TO CONNECT")
            || test_response.contains("BUS INIT: ERROR")
            || test_response.contains("NO DATA")
            || test_response.contains("CAN ERROR")
            || test_response.trim().is_empty()
        {
            return Err(format!(
                "ECU communication check failed (ignition off or unplugged): {}",
                test_response.trim()
            ));
        }

        Ok(adapter)
    }

    fn send_request(&self, command: &[u8]) -> Result<String, String> {
        let mut conn = self.connection.lock().map_err(|_| "Failed to lock mutex")?;
        conn.write_all(command).map_err(|e| e.to_string())?;

        let mut response = Vec::new();
        let mut byte_buf = [0u8; 1];

        // Keep reading until the ELM327 prompt character '>' is encountered
        loop {
            match conn.read(&mut byte_buf) {
                Ok(1) => {
                    if byte_buf[0] == b'>' {
                        break;
                    }
                    response.push(byte_buf[0]);
                }
                Ok(_) => break,
                Err(e) => {
                    // Stop on timeout or stream errors
                    if !response.is_empty() {
                        break;
                    }
                    return Err(e.to_string());
                }
            }
        }

        Ok(String::from_utf8_lossy(&response).to_string())
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
        let rpm = self.read_engine_rpm()?;
        let speed = self.read_vehicle_speed()?;
        Ok(calculate_gear_from_ratio(rpm, speed))
    }
}

// --- UNIT TESTS ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_engine_rpm() {
        assert_eq!(parse_engine_rpm("41 0C 1A F8\r\n"), 1726);
        assert_eq!(parse_engine_rpm("410C1AF8"), 1726);
        assert_eq!(parse_engine_rpm("SEARCHING..."), 0);
        assert_eq!(parse_engine_rpm("NODATA"), 0);
    }

    #[test]
    fn test_parse_vehicle_speed() {
        assert_eq!(parse_vehicle_speed("41 0D 32\r\n"), 50);
        assert_eq!(parse_vehicle_speed("410D32"), 50);
        assert_eq!(parse_vehicle_speed("ERROR"), 0);
    }

    #[test]
    fn test_parse_oil_temp() {
        assert_eq!(parse_oil_temp("41 5C 5A\r\n"), 50.0);
        assert_eq!(parse_oil_temp("41 5C 28"), 0.0);
    }

    #[test]
    fn test_parse_error_code() {
        assert_eq!(parse_error_code("43 01 33\r\n"), "P0133");
        assert_eq!(parse_error_code("43 00 00\r\n"), "NONE");
        assert_eq!(parse_error_code("NO DATA"), "UNKNOWN");
    }
}