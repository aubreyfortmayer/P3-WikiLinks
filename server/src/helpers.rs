use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};

pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::new()
        .map_err(|never| match never {})
        .boxed()
}

pub fn full<T>(chunk: T) -> BoxBody<Bytes, hyper::Error>
where T: Into<Bytes>,
{
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}