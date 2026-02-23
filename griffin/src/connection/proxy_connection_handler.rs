use griffin_core::forward;
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
    fn handle(
        &self,
        stream: tokio::net::TcpStream,
        metrics: Arc<Metrics>,
        authority: Authority,
        _: &str,
    ) {
        // your existing logic here
        let io = TokioIo::new(stream);
        let metrics = metrics.clone();
        let authority = authority.clone();

        tokio::spawn(async move {
            let svc =
                tower::service_fn(move |req| forward(req, authority.clone(), metrics.clone()));

            let svc = TowerToHyperService::new(svc);

            if let Err(err) = AutoBuilder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                eprintln!("proxy error: {:?}", err);
            }
        });
    }
}
// pub fn run_proxy(
//     stream: tokio::net::TcpStream,
//     metrics: Arc<Metrics>,
//     forward_authority: Authority,
// ) {
//     // Placeholder for potential future implementation
//     let io = TokioIo::new(stream);
//     let metrics = metrics.clone();
//     let forward_authority = forward_authority.clone();
//     tokio::task::spawn(async move {
//         let forward_authority = forward_authority.clone();
//         let metrics = metrics.clone();
//         let svc = tower::service_fn(move |req| {
//             let forward_authority = forward_authority.clone();
//             let metrics = metrics.clone();
//             forward(req, forward_authority, metrics)
//         });
//         let svc = TowerToHyperService::new(svc);
//         if let Err(err) = AutoBuilder::new(TokioExecutor::new())
//             .serve_connection(io, svc)
//             .await
//         {
//             eprintln!("Error serving connection: {:?}", err);
//         }
//     });
// }
