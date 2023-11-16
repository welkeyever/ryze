mod uppercase_middleware;

use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::Future;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use route_recognizer::{Params, Router};
use tokio::sync::Mutex;

type BoxFut = Pin<Box<dyn Future<Output=Result<Response<Body>, hyper::Error>> + Send>>;
type Handler = Arc<dyn Fn(Request<Body>) -> BoxFut + Send + Sync>;

fn index(_req: Request<Body>) -> BoxFut {
    Box::pin(async {
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

    let router: Router<Handler> = Router::new();
    let router = Arc::new(Mutex::new(router));

    router.lock().await.add("/", Arc::new(index));
    router.lock().await.add("/hello", Arc::new(hello));

    let make_svc = make_service_fn(move |_| {
        let router = Arc::clone(&router);
        async {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let router = Arc::clone(&router);
                async move {
                    let path = req.uri().path().to_owned();
                    let router = router.lock().await;
                    match router.recognize(&path) {
                        Ok(matched) => (matched.handler)(req).await,
                        Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()),
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