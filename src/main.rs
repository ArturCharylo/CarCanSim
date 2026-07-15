use axum::{routing::get, Router};
use prometheus::{Encoder, Gauge, TextEncoder};
use lazy_static::lazy_static;
use rand::{RngExt};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

lazy_static! {
    static ref VEHICLE_SPEED: Gauge = Gauge::new("VEHICLE_SPEED", "Current Vehicle Speed").expect("Failed to create Vehicle speed gauge");
    static ref ENGINE_RPM: Gauge = Gauge::new("ENGINE_RPM", "Current Engine RPM").expect("Failed to create Engine RPM gauge");
    static ref OIL_TEMP: Gauge = Gauge::new("OIL_TEMP", "Current Oil Temperature").expect("Failed to create oil temp gauge");
    static ref ERR_CODE: Gauge = Gauge::new("ERR_CODE", "Current error code").expect("Failed to create Error Code gauge");
}

#[tokio::main]
async fn main() {
    prometheus::register(Box::new(VEHICLE_SPEED.clone())).unwrap();
    prometheus::register(Box::new(ENGINE_RPM.clone())).unwrap();
    prometheus::register(Box::new(OIL_TEMP.clone())).unwrap();
    prometheus::register(Box::new(ERR_CODE.clone())).unwrap();

    tokio::spawn(async move {
        loop {
            // By wrapping this in a new scope, the non-Send `rng` is dropped 
            // before the asynchronous sleep, making the future safe to send across threads.
            {
                let mut rng = rand::rng();

                let speed: f64 = rng.random_range(0.0..100.0);
                VEHICLE_SPEED.set(speed);
                
                let rpm: f64 = rng.random_range(800.0..8500.0);
                ENGINE_RPM.set(rpm);

                let oil: f64 = rng.random_range(80.0..110.0);
                OIL_TEMP.set(oil);

                let err: f64 = rng.random_range(0.0..100.0);
                ERR_CODE.set(err);
            } // `rng` is dropped here

            sleep(Duration::from_secs(1)).await;
        }
    });

    let app = Router::new().route("/metrics", get(metrics_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Running on port http://localhost:{}/metrics", addr.port());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

async fn metrics_handler() -> String{
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    
    // Encode the gathered metrics into the Prometheus text format
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    String::from_utf8(buffer).unwrap()
}