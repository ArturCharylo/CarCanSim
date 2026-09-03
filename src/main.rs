mod metrics;

use axum::{middleware, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use car_can_sim::obd::{
    hardware::{calculate_gear_from_ratio, HardwareAdapter},
    simulator::Simulator,
    ObdInterface,
};

use metrics::{
    health_handler, metrics_handler, register_metrics, track_metrics_middleware, CURRENT_GEAR,
    ENGINE_RPM, ERR_CODE, OIL_TEMP, VEHICLE_SPEED,
};

#[tokio::main]
async fn main() {
    // Initialize Prometheus metrics
    register_metrics();

    let obd_mode = std::env::var("OBD_MODE").unwrap_or_else(|_| "auto".to_string());
    let port = std::env::var("OBD_PORT").unwrap_or_else(|_| "COM3".to_string());

    // Setup interface with auto-fallback support
    let obd_interface: Arc<Box<dyn ObdInterface>> = match obd_mode.as_str() {
        "hardware" => {
            println!("Explicit hardware mode requested on port: {}", port);
            let adapter =
                HardwareAdapter::new(&port).expect("Failed to initialize OBD-II hardware adapter");
            Arc::new(Box::new(adapter))
        }
        "simulator" => {
            println!("Explicit simulator mode requested.");
            Arc::new(Box::new(Simulator::new()))
        }
        _ => {
            // Default "auto" mode: try connecting to hardware, fallback to simulator
            println!("Attempting connection to OBD-II adapter on {}...", port);
            match HardwareAdapter::new(&port) {
                Ok(adapter) => {
                    println!("Successfully connected to vehicle via {}", port);
                    Arc::new(Box::new(adapter))
                }
                Err(e) => {
                    eprintln!("Hardware connection unavailable ({}), falling back to simulator.", e);
                    Arc::new(Box::new(Simulator::new()))
                }
            }
        }
    };

    // Initial sanity check print
    match obd_interface.read_engine_rpm() {
        Ok(rpm) => println!("Initial test RPM read: {}", rpm),
        Err(e) => println!("Initial test RPM read failed: {}", e),
    }

    // Run blocking serial/hardware polling on a dedicated OS thread to prevent blocking Tokio workers
    let interface_clone = Arc::clone(&obd_interface);
    std::thread::spawn(move || {
        loop {
            // Read speed
            let speed = match interface_clone.read_vehicle_speed() {
                Ok(val) => {
                    VEHICLE_SPEED.set(val as f64);
                    Some(val)
                }
                Err(e) => {
                    eprintln!("Error reading speed: {}", e);
                    None
                }
            };

            // Read RPM
            let rpm = match interface_clone.read_engine_rpm() {
                Ok(val) => {
                    ENGINE_RPM.set(val as f64);
                    Some(val)
                }
                Err(e) => {
                    eprintln!("Error reading RPM: {}", e);
                    None
                }
            };

            // Read oil temperature
            match interface_clone.read_oil_temp() {
                Ok(temp) => OIL_TEMP.set(temp as f64),
                Err(e) => eprintln!("Error reading oil temp: {}", e),
            }

            // Read error codes
            match interface_clone.read_error_code() {
                Ok(code) => {
                    if code == "NONE" || code == "UNKNOWN" {
                        ERR_CODE.set(0.0);
                    } else {
                        // Diagnostic Trouble Code (DTC) detected
                        ERR_CODE.set(1.0);
                        println!("Diagnostic Trouble Code detected: {}", code);
                    }
                }
                Err(e) => eprintln!("Error reading error code: {}", e),
            }

            // Calculate current gear from already queried metrics instead of sending extra requests
            if let (Some(r), Some(s)) = (rpm, speed) {
                let gear = calculate_gear_from_ratio(r, s);
                CURRENT_GEAR.set(gear as f64);
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });

    // Setup HTTP server routes
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .layer(middleware::from_fn(track_metrics_middleware));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Running on http://localhost:{}/metrics", addr.port());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}