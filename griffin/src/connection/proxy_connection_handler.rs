use griffin_core::proxy_request;
use griffin_core::telemetry::metrics::Metrics;
use http::uri::Authority;
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    service::TowerToHyperService,
};
use std::sync::Arc;

use crate::connection::connection_handler::ConnectionHandler;
#[derive(Clone)]
pub struct ProxyConnectionHandler;

impl ConnectionHandler for ProxyConnectionHandler {
    async fn serve_connection(
        &self,
        stream: tokio::net::TcpStream,
        metrics: Arc<Metrics>,
        authority: Authority,
    ) {
        let io = TokioIo::new(stream);
        let svc =
            tower::service_fn(move |req| proxy_request(req, authority.clone(), metrics.clone()));
        let svc = TowerToHyperService::new(svc);
        if let Err(err) = AutoBuilder::new(TokioExecutor::new())
            .serve_connection(io, svc)
            .await
        {
            eprintln!("proxy error: {:?}", err);
        }
    }
}
