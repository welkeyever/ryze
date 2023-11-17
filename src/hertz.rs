use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use futures::future::Future;
use std::pin::Pin;
use route_recognizer::{Params, Router};

type BoxFut = Pin<Box<dyn Future<Output=Result<Response<Body>, hyper::Error>> + Send>>;
type Handler = Arc<dyn Fn(Request<Body>) -> BoxFut + Send + Sync>;

pub struct Hertz {
    router: Router<Handler>,
    middleware: Vec<Handler>,
}

impl Hertz {
    // create an instance of Hertz
    pub fn new() -> Self {
        Hertz {
            router: Router::new(),
            middleware: Vec::new(),
        }
    }

    // register a middleware
    pub fn use_fn(&mut self, middleware: Handler) {
        self.middleware.push(middleware);
    }

    // register a route handler for GET requests
    pub fn get(&mut self, path: &str, handler: Handler) {
        let mut combined_handler = handler.clone();
        for middleware in self.middleware.iter().rev() {
            let next_handler = combined_handler.clone();
            let this_middleware = middleware.clone();

            combined_handler = Arc::new(move |req: Request<Body>| {
                let next_handler = next_handler.clone();
                Box::pin(async move {
                    // First, run this middleware
                    let _ = (this_middleware)(req.clone()).await?;
                    // Then, run the next handler (either another middleware or the final handler).
                    next_handler(req).await
                })
            });
        }
        self.router.add(path, combined_handler);
    }

    // start the server
    // #[tokio::main]
    pub async fn spin(&self, addr: SocketAddr) -> hyper::Result<()> {
        let router = Arc::new(Mutex::new(self.router.clone()));

        let make_svc = make_service_fn(move |_| {
            let inner_router = Arc::clone(&router);
            async {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let router = Arc::clone(&inner_router);
                    async move {
                        match router.lock().await.recognize(req.uri().path()) {
                            Ok(matched) => (matched.handler)(req).await,
                            Err(_) => Ok(Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("404 not found")).unwrap()),
                        }
                    }
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);
        println!("Listening on http://{}", addr);
        server.await
    }
}