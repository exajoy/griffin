use std::convert::Infallible;

use async_stream::try_stream;
use bytes::Bytes;
use http_body_util::{BodyExt, Full, StreamBody};
use once_cell::sync::Lazy;
use prometheus::{
    CounterVec, Encoder, HistogramVec, TextEncoder, register_counter_vec, register_histogram_vec,
};

use crate::core::stream_response::StreamResponse;
//
// 1. METRICS ARE REGISTERED *ONCE* HERE
//
pub static REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests handled",
        &["method", "path"]
    )
    .expect("metric already registered")
});

pub static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "http_request_duration_seconds",
        "Request duration in seconds",
        &["method", "path"]
    )
    .expect("metric already registered")
});
#[derive(Clone)]
pub struct Metrics;

impl Metrics {
    pub fn new() -> Self {
        Metrics
    }

    pub fn requests_total(&self) -> &CounterVec {
        &REQUESTS_TOTAL
    }

    pub fn request_duration(&self) -> &HistogramVec {
        &REQUEST_DURATION
    }

    pub fn render(&self) -> StreamResponse {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let body = Full::<Bytes>::from(buffer);
        from_full_bytes(body)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

pub fn from_full_bytes(mut body: Full<Bytes>) -> StreamResponse {
    let forward_stream = try_stream! {
        while let Some(frame) = body.frame().await {
            let frame = frame.map_err(|e: Infallible| -> hyper::Error { match e {} })?;
            yield frame;
        }
    };

    StreamResponse::new(StreamBody::new(Box::pin(forward_stream)))
}
