use async_stream::try_stream;
use bytes::Bytes;
use griffin_core::core::stream_response::StreamResponse;
use http::Request;
use http_body_util::Full;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Incoming;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

use griffin_test::test_support::{
    greeter::hello_world::{HelloReply, HelloRequest},
    preparation::run_intergration,
    utils::{collect_messages, message_to_frame},
};

use http::Response;
use tower::BoxError;

fn incoming_to_stream_body(res: Response<Incoming>) -> StreamResponse {
    let forward_stream = try_stream! {
            let mut incoming = res.into_body();
        while let Some(frame) = incoming.frame().await {
            let frame = frame?;
            yield frame;
        }
    };
    StreamResponse::new(StreamBody::new(Box::pin(forward_stream)))
}

#[tokio::test]
async fn test_grpc_web_unary_call() -> Result<(), BoxError> {
    run_intergration(async move |proxy_address| {
        let url = format!("http://{}/helloworld.Greeter/SayHello", proxy_address);
        let req_msg = HelloRequest {
            name: "Alice".to_string(),
        };

        let req = Request::post(url)
            .header("content-type", "application/grpc")
            .body(Full::<Bytes>::from(message_to_frame(&req_msg).freeze()))
            .unwrap();

        let client = Client::builder(TokioExecutor::new()).build_http();

        let res = client.request(req).await.unwrap();

        // let res = res.map(|body| incoming_to_stream_body(body));
        let res = incoming_to_stream_body(res);
        assert_eq!(res.status(), 200);
        let body = res.into_body();
        let messages: Vec<HelloReply> = collect_messages(body).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages.first().unwrap().message, "Hello Alice!");

        Ok(())
    })
    .await
}
