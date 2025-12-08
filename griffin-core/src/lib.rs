use bytes::Bytes;
use futures_core::Stream;
use http::{Request, Response, Uri, header::CONTENT_TYPE, uri::Authority};
use http_body::Frame;
use http_body_util::StreamBody;
use hyper::client::conn::http2;
use hyper_util::rt::{TokioExecutor, TokioIo};
use scopeguard::defer;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tower::BoxError;

use crate::core::grpc_kind::GrpcKind;
use crate::telemetry::metrics::Metrics;

pub mod core;
pub mod telemetry;
pub mod trailers;

pub async fn forward<B>(
    req: Request<B>,
    authority: Authority,
    metrics: Arc<Metrics>,
) -> Result<
    Response<StreamBody<impl Stream<Item = Result<Frame<Bytes>, hyper::Error>> + Send>>,
    BoxError,
>
where
    B: hyper::body::Body<Data = Bytes> + Send + 'static + Unpin,
    B::Error: Into<BoxError>,
{
    //[START] switch endpoint
    let (mut parts, req_body) = req.into_parts();
    parts
        .headers
        .insert(hyper::header::HOST, authority.as_str().parse()?);

    let path = parts.uri.path().to_string();

    // Early exit for /metrics
    if path == "/metrics" {
        return Ok(metrics.render());
    }
    let start = Instant::now();
    defer!({
        let elapsed = start.elapsed().as_secs_f64();
        metrics
            .requests_total()
            .with_label_values(&[&"POST", &path.as_str()])
            .inc();
        metrics
            .request_duration()
            .with_label_values(&[&"POST", &path.as_str()])
            .observe(elapsed);
    });
    let url = format!("http://{}{}", authority.as_ref(), path);

    parts.uri = url.parse::<Uri>()?;

    //[END] switch endpoint

    let stream = TcpStream::connect(authority.to_string()).await?;
    let io = TokioIo::new(stream);

    let exec = TokioExecutor::new();
    let (sender, conn): (
        http2::SendRequest<_>,
        http2::Connection<TokioIo<TcpStream>, _, TokioExecutor>,
    ) = http2::Builder::new(exec).handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });
    let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .ok_or("Missing Content-Type header")?
        .clone();
    let req = Request::from_parts(parts, req_body);
    GrpcKind::from_content_type(&content_type)
        .ok_or("Unsupported Content-Type header")?
        .forward(sender, req)
        .await
}
