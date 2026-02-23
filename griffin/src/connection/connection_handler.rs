use griffin_core::telemetry::metrics::Metrics;
use http::uri::Authority;
use std::sync::Arc;

pub trait ConnectionHandler: Send + Sync + 'static {
    fn handle(
        &self,
        stream: tokio::net::TcpStream,
        metrics: Arc<Metrics>,
        authority: Authority,
        proxy_address: &str,
    );
}
