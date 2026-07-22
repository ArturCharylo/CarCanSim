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
        let _raw_response = self.send_request(b"010C\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        // TODO: Implement parsing logic to convert the hex string response into a u32 value
        // Returning a mock value to satisfy the compiler for now
        Ok(0)
    }

    fn read_vehicle_speed(&self) -> Result<u8, ObdError> {
        let _raw_response = self.send_request(b"010D\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        // TODO: Implement parsing logic
        Ok(0)
    }

    fn read_oil_temp(&self) -> Result<f32, ObdError> {
        let _raw_response = self.send_request(b"015C\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        // TODO: Implement parsing logic
        Ok(0.0)
    }

    fn read_error_code(&self) -> Result<u8, ObdError> {
        let _raw_response = self.send_request(b"03\r").map_err(|_| {
            ObdError::NotImplemented("Hardware connection error".to_string())
        })?;
        
        // TODO: Implement parsing logic for Diagnostic Trouble Codes
        Ok(0)
    }
}