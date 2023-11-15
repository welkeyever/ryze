use std::net::SocketAddr;

use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use uppercase_middleware::UppercaseMiddleware;

mod uppercase_middleware;

async fn handler(req: Request<Body>) -> Result<Response<Body>, Error> {  // 注意这里的错误类型 Error
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            if let Some(query) = req.uri().query() {
                return Ok(Response::new(Body::from(query.to_owned())));
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

    let make_svc = make_service_fn(|_conn| {
        let svc = UppercaseMiddleware::new(service_fn(handler));
        async { Ok::<_, hyper::Error>(svc) } // 注意这里的错误类型是 hyper::Error
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}