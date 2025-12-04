use futures_util::StreamExt;

use hyper_util::rt::TokioExecutor;
use tonic::Request;
use tonic_web::GrpcWebClientLayer;

use griffin_test::test_support::{
    greeter::hello_world::{HelloRequest, greeter_client::GreeterClient},
    preparation::run_intergration,
};
use tower::BoxError;

#[tokio::test]
async fn test_grpc_web_server_streaming_call() -> Result<(), BoxError> {
    run_intergration(async move |proxy_address| {
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();

        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .service(client);

        let mut client =
            GreeterClient::with_origin(svc, format!("http://{}", proxy_address).try_into()?);

        let request = HelloRequest {
            name: "Tonic".into(),
        };

        let res = client
            .say_hello_stream(Request::new(request))
            .await
            .unwrap();

        let mut stream = res.into_inner();
        let mut replies = Vec::<String>::new();
        while let Some(resp) = stream.next().await {
            match resp {
                Ok(msg) => {
                    replies.push(msg.message.clone());
                }
                Err(e) => {
                    break;
                }
            }
        }
        assert_eq!(
            replies,
            vec!["first ok".to_string(), "second ok".to_string(),]
        );

        Ok(())
    })
    .await
}
