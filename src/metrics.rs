use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode
};
use lazy_static::lazy_static;
use prometheus::{Gauge, CounterVec, Opts, TextEncoder};

lazy_static! {
    pub static ref VEHICLE_SPEED: Gauge = Gauge::new("VEHICLE_SPEED", "Current Vehicle Speed")
        .expect("Failed to create Vehicle speed gauge");
    pub static ref ENGINE_RPM: Gauge =
        Gauge::new("ENGINE_RPM", "Current Engine RPM").expect("Failed to create Engine RPM gauge");
    pub static ref OIL_TEMP: Gauge =
        Gauge::new("OIL_TEMP", "Current Oil Temperature").expect("Failed to create oil temp gauge");
    pub static ref ERR_CODE: Gauge =
        Gauge::new("ERR_CODE", "Current error code").expect("Failed to create Error Code gauge");
    pub static ref CURRENT_GEAR: Gauge =
        Gauge::new("CURRENT_GEAR", "Current gear").expect("Failed to read current gear");

    // Metric for counting response statuses
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = 
        CounterVec::new(Opts::new("http_requests_total", "Total number of HTTP requests processed"),
        &["path", "status"]
    ).expect("Failed to create HTTP requests counter");
}

// Function to register all metrics
pub fn register_metrics() {
    prometheus::register(Box::new(VEHICLE_SPEED.clone())).unwrap();
    prometheus::register(Box::new(ENGINE_RPM.clone())).unwrap();
    prometheus::register(Box::new(OIL_TEMP.clone())).unwrap();
    prometheus::register(Box::new(ERR_CODE.clone())).unwrap();
    prometheus::register(Box::new(CURRENT_GEAR.clone())).unwrap();
    prometheus::register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
}

// Middleware to track HTTP status codes
pub async fn track_metrics_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let response = next.run(req).await;
    let status = response.status();

    let status_family = match status.as_u16() {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    };

    // Increment counter for the matching path and status group
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&path, status_family])
        .inc();

    response
}

// Encode the gathered metrics into the Prometheus text format
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = String::new();

    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();

    buffer
}

// Simple health check endpoint for liveness probes
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}