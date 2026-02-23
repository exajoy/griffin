use bytes::Bytes;
use http::{HeaderValue, Request, Response};
use http_body::Frame;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::body::Incoming;

use crate::trailers::Trailers;
pub struct GrpcKindWeb;
impl GrpcKindWeb {
    pub fn modify_request<B>(&self, req: &mut Request<B>)
    where
        B: hyper::body::Body,
    {
        req.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/grpc"),
        );
        req.headers_mut().remove(hyper::header::CONTENT_LENGTH);
    }

    pub fn modify_response(
        &self,
        res: Response<Incoming>,
    ) -> Response<BoxBody<Bytes, hyper::Error>> {
        let (parts, body) = res.into_parts();
        let transformed = body
            .map_frame(|frame| {
                if let Some(trailers) = frame.trailers_ref() {
                    let t = Trailers::new(trailers.clone());
                    Frame::data(t.into_to_frame())
                } else {
                    frame
                }
            })
            .boxed();

        let mut res = Response::from_parts(parts, transformed);
        res.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/grpc-web+proto"),
        );
        res
    }
}
