use std::io::Read;

use async_trait::async_trait;
use bytes::{Buf, Bytes};
use hyper::body;
use hyper::Body;
use hyper::Response;
use hyper::service::Service;

pub struct UppercaseMiddleware<S> {
    inner: S,
}

impl<S> UppercaseMiddleware<S> {
    pub fn new(inner: S) -> Self {
        UppercaseMiddleware {
            inner
        }
    }
}

#[async_trait]
impl<S, ReqBody> Service<hyper::Request<ReqBody>> for UppercaseMiddleware<S>
    where
        S: Service<hyper::Request<ReqBody>, Response=Response<Body>, Error=hyper::Error> + Send + Clone + 'static,
        S::Future: Send + 'static,
        ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<ReqBody>) -> Self::Future {
        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let (parts, body) = res.into_parts();

            let whole_body = body::to_bytes(body).await?;
            let full_body = whole_body.reader().bytes().collect::<Result<Vec<u8>, _>>().unwrap();
            let uppercased = full_body.iter().map(|byte| byte.to_ascii_uppercase()).collect::<Vec<u8>>();

            let body = Body::from(Bytes::from(uppercased));
            let response = Response::from_parts(parts, body);

            Ok::<_, hyper::Error>(response)
        })
    }
}