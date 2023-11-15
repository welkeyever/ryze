extern crate route_recognizer;
use std::net::SocketAddr;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use route_recognizer::{Router, Match};

async fn handler(req: Request<Body>, matched: Match<&String>) -> Result<Response<Body>, Error> {
    match (req.method(), matched.handler.as_str()) {
        (&Method::GET, "/") => {
            if let Some(query) = req.uri().query() {
                return Ok(Response::new(Body::from(query.to_owned())))
            }
            let mut default = Response::default();
            *default.body_mut() = Body::from("ok");
            Ok(default)
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
    let mut router = Router::new();
    router.add("/", "/".to_owned());

    let make_svc = make_service_fn(move |_conn| {
        // Clone router here
        let router = router.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let path = req.uri().path().to_owned();
                // Clone router again inside async block
                let router = router.clone();
                // Handler needs to be async block instead of a function
                async move {
                    let matched = router.recognize(&path).unwrap();
                    handler(req, matched).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}