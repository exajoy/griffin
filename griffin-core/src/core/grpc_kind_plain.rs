use bytes::Bytes;
use http::Response;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::body::Incoming;

pub struct GrpcKindPlain;

impl GrpcKindPlain {
    pub fn modify_response(
        &self,
        res: Response<Incoming>,
    ) -> Response<BoxBody<Bytes, hyper::Error>> {
        res.map(|body| body.boxed())
    }
}
