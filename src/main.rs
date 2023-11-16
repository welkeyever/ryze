use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::Future;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use route_recognizer::{Params, Router};
use tokio::sync::Mutex;

mod uppercase_middleware;

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
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let mut router: Router<Handler> = Router::new();
    router.add("/", Arc::new(index));
    router.add("/hello", Arc::new(hello));

    let router = Arc::new(Mutex::new(router));

    let make_svc = make_service_fn(move |_| {
        let router = Arc::clone(&router);
        async {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let router = Arc::clone(&router);
                async move {
                    match router.lock().await.recognize(req.uri().path()) {
                        Ok(matched) => (matched.handler)(req).await,
                        Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("404 not found")).unwrap()),
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}