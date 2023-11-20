use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use route_recognizer::Router;
use tokio::sync::Mutex;

pub struct RequestContext {
    pub req: Request<Body>,
    pub resp: Response<Body>,
    middlewares: Vec<Handler>,
    middleware_index: usize,
    // Add your CX here
    //'cx: CX
}

// Handler type
pub type Handler = Arc<dyn Fn(&mut RequestContext) + Send + Sync>;

// Implementing methods for RequestContext
impl RequestContext {
    pub fn next(&mut self) {
        if self.middleware_index < self.middlewares.len() {
            let middleware = self.middlewares[self.middleware_index].clone();
            self.middleware_index += 1;
            middleware(self);
        }
    }
}

// Definition for Main Hertz Struct
pub struct Hertz {
    router: Arc<Mutex<Router<Handler>>>,
    middlewares: Vec<Handler>,
}

// Implementing methods for Hertz
impl Hertz {
    // Creates a new instance of Hertz
    pub fn new() -> Self {
        Hertz {
            router: Arc::new(Mutex::new(Router::new())),
            middlewares: Vec::new(),
        }
    }

    // Adds a middleware to the Hertz instance
    pub fn use_fn(&mut self, middleware: Handler) {
        self.middlewares.push(middleware);
    }

    // Adds a route to the Hertz instance
    pub async fn get(&self, path: &str, handler: Handler) {
        let mut router = self.router.lock().await;
        router.add(path, handler);
    }

    // Starts the Hertz instance
    pub async fn spin(self, addr: SocketAddr) -> hyper::Result<()> {
        let router = Arc::clone(&self.router);
        let middlewares = Arc::new(self.middlewares);

        let make_svc = make_service_fn(move |_| {
            let router = Arc::clone(&router);
            let middlewares = Arc::clone(&middlewares);
            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let resp = Response::new(Body::empty());
                    let router = Arc::clone(&router);
                    let middlewares = middlewares.clone();
                    async move {
                        let middlewares = middlewares.clone();
                        match router.lock().await.recognize(req.uri().path()) {
                            Ok(matched) => {
                                let mut middlewares = middlewares.to_vec();
                                middlewares.push(matched.handler.clone());

                                let mut ctx = RequestContext {
                                    req,
                                    resp,
                                    middleware_index: 0,
                                    middlewares,
                                };
                                ctx.next();  // run through middlewares
                                Ok::<_, hyper::Error>(ctx.resp)
                            }
                            Err(err) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from(format!("{}",router))).unwrap())
                        }
                    }
                }))
            }
        });

        println!(r#"running server on "{}""#, addr);
        Server::bind(&addr).serve(make_svc).await?;  // Run the server
        Ok(())
    }
}