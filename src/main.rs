use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::Future;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use route_recognizer::{Params, Router};
use tokio::sync::Mutex;

mod uppercase_middleware;
mod hertz;

type BoxFut = Pin<Box<dyn Future<Output=Result<Response<Body>, hyper::Error>> + Send>>;
type Handler = Arc<dyn Fn(Request<Body>) -> BoxFut + Send + Sync>;

fn index(req: Request<Body>) -> BoxFut {
    Box::pin(async {
        let (header, _) = req.into_parts();
        if let Some(query) = header.uri.query() {
            let response = Response::new(Body::from(query.chars().map(|b| b.to_ascii_uppercase()).collect::<String>()));
            return Ok(response);
        }

        let response = Response::new(Body::from("Hello, world!"));
        Ok(response)
    })
}

fn hello(_req: Request<Body>) -> BoxFut {
    Box::pin(async {
        let response = Response::new(Body::from("Pong!"));
        Ok(response)
    })
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let mut h = hertz::Hertz::new();
    h.get("/ping", Arc::new(hello));
    h.spin(SocketAddr::from(([127, 0, 0, 1], 8000))).await
}