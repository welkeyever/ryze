use std::net::SocketAddr;
use std::sync::Arc;
use futures::future::BoxFuture;
use futures::FutureExt;

use hyper::Body;

use hertz::{Hertz, RequestContext};

mod hertz;

// Index handler
// async fn index(ctx: &mut RequestContext) {
//     let query = ctx.req.uri().query().unwrap_or("");
//     let body = query.chars().map(|b| b.to_ascii_uppercase()).collect::<String>();
//     *ctx.resp.body_mut() = Body::from(body);
// }

// Hello (or ping) handler
fn hello(ctx: &mut RequestContext) -> BoxFuture<'_, ()> {
    (async move {
        *ctx.resp.body_mut() = Body::from("Pong!");
    }).boxed()
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let mut h = Hertz::new();

    // middleware 0
    // h.use_fn(Arc::new(|req_ctx| {
    //     println!("pre-handle 0");
    //     req_ctx.next();
    //     println!("post-handle 0");
    // }));
    //
    // // middleware 1
    // h.use_fn(Arc::new(|req_ctx| {
    //     println!("pre-handle 1");
    //     req_ctx.next();
    //     println!("post-handle 1");
    // }));

    // route registry
    // h.get("/query", Arc::new(index)).await;
    h.get("/ping", Arc::new(hello)).await;

    // run forever
    h.spin(SocketAddr::from(([127, 0, 0, 1], 8000))).await
}