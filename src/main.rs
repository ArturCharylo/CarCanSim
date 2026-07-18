use axum::{Router, routing::get};
use lazy_static::lazy_static;
use prometheus::{Encoder, Gauge, TextEncoder};
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};

use car_can_sim::obd::{hardware::HardwareAdapter, simulator::Simulator, ObdInterface};

lazy_static! {
    static ref VEHICLE_SPEED: Gauge = Gauge::new("VEHICLE_SPEED", "Current Vehicle Speed")
        .expect("Failed to create Vehicle speed gauge");
    static ref ENGINE_RPM: Gauge =
        Gauge::new("ENGINE_RPM", "Current Engine RPM").expect("Failed to create Engine RPM gauge");
    // We will keep OIL_TEMP and ERR_CODE static for now as they are not in the new interface
    static ref OIL_TEMP: Gauge =
        Gauge::new("OIL_TEMP", "Current Oil Temperature").expect("Failed to create oil temp gauge");
    static ref ERR_CODE: Gauge =
        Gauge::new("ERR_CODE", "Current error code").expect("Failed to create Error Code gauge");
}

#[tokio::main]
async fn main() {
    prometheus::register(Box::new(VEHICLE_SPEED.clone())).unwrap();
    prometheus::register(Box::new(ENGINE_RPM.clone())).unwrap();
    prometheus::register(Box::new(OIL_TEMP.clone())).unwrap();
    prometheus::register(Box::new(ERR_CODE.clone())).unwrap();

    let obd_mode = std::env::var("OBD_MODE").unwrap_or_else(|_| "simulator".to_string());

    let obd_interface: Box<dyn ObdInterface> = if obd_mode == "hardware" {
        Box::new(HardwareAdapter)
    } else {
        Box::new(Simulator)
    };

    // Test print
    match obd_interface.read_engine_rpm() {
        Ok(rpm) => println!("Initial test RPM read: {}", rpm),
        Err(e) => println!("Initial test RPM read failed: {}", e),
    }

    tokio::spawn(async move {
        loop {
            // Read from interface
            match obd_interface.read_vehicle_speed() {
                Ok(speed) => VEHICLE_SPEED.set(speed as f64),
                Err(e) => println!("Error reading speed: {}", e),
            }

            match obd_interface.read_engine_rpm() {
                Ok(rpm) => ENGINE_RPM.set(rpm as f64),
                Err(e) => println!("Error reading RPM: {}", e),
            }

            // Keep dummy data for others
            OIL_TEMP.set(90.0);
            ERR_CODE.set(0.0);

            sleep(Duration::from_secs(1)).await;
        }
    });

    let app = Router::new().route("/metrics", get(metrics_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Running on port http://localhost:{}/metrics", addr.port());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    // Encode the gathered metrics into the Prometheus text format
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}
