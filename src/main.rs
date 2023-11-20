use std::net::SocketAddr;
use std::sync::Arc;

use hyper::Body;

use hertz::{Handler, Hertz, RequestContext};

mod hertz;

// Index handler
fn index(ctx: &mut RequestContext) {
    let query = ctx.req.uri().query().unwrap_or("");
    let body = query.chars().map(|b| b.to_ascii_uppercase()).collect::<String>();
    *ctx.resp.body_mut() = Body::from(body);
}

// Hello (or ping) handler
fn hello(ctx: &mut RequestContext) {
    *ctx.resp.body_mut() = Body::from("Pong!");
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let mut h = Hertz::new();
    h.get("/query", Arc::new(index));
    h.get("/ping", Arc::new(hello));
    h.spin(SocketAddr::from(([127, 0, 0, 1], 8000))).await
}