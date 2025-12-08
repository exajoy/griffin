use griffin_core::forward;
use griffin_core::telemetry::metrics::Metrics;
use http::uri::Authority;
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    service::TowerToHyperService,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower::BoxError;

pub mod args;
pub mod config;
pub mod listener;

pub async fn start_proxy(
    listener: TcpListener,
    forward_authority: String,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), BoxError> {
    let forward_authority = Authority::from_str(&forward_authority)?;
    let metrics = Arc::new(Metrics::new());
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _)) => {
                        run_proxy(stream, metrics.clone(), forward_authority.clone());
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {:?}", e);
                    }
                }
            }

             _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    println!("Proxy shutdown signal received");
                    break;
                }
            }
        }
    }
    Ok(())
}

pub fn run_proxy(
    stream: tokio::net::TcpStream,
    metrics: Arc<Metrics>,
    forward_authority: Authority,
) {
    // Placeholder for potential future implementation
    let io = TokioIo::new(stream);
    let metrics = metrics.clone();
    let forward_authority = forward_authority.clone();
    tokio::task::spawn(async move {
        let forward_authority = forward_authority.clone();
        let metrics = metrics.clone();
        let svc = tower::service_fn(move |req| {
            let forward_authority = forward_authority.clone();
            let metrics = metrics.clone();
            forward(req, forward_authority, metrics)
        });
        let svc = TowerToHyperService::new(svc);
        if let Err(err) = AutoBuilder::new(TokioExecutor::new())
            .serve_connection(io, svc)
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    });
}
