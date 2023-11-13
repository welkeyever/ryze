// 需要使用hyper库的一些组件，如Body, Request, Response, Server等
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use std::net::SocketAddr;

// 处理收到的HTTP请求
async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // 根据请求的方法和路径进行匹配
    match (req.method(), req.uri().path()) {
        // 如果是 GET 请求且请求路径为 "/"
        (&Method::GET, "/") => {
            Ok(Response::new(Body::from("Hello, World!")))
        },
        // 对于其他的请求，返回"not found"
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        },
    }
}

#[tokio::main]
// 声明一个异步的 main 函数，这个函数返回一个Result类型
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 设定服务的地址和端口
    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));

    // 创建一个新的服务，这个服务会在每次连接到来时创建一个 handler 实例
    let make_service = make_service_fn(|_conn| {
        async {
            Ok::<_, Infallible>(service_fn(handler))
        }
    });

    // 使用地址绑定服务，并在每个请求到来时调用服务
    let server = Server::bind(&addr).serve(make_service);

    // 输出服务监听的地址
    println!("Listening on http://{}", addr);

    // 运行服务并处理错误
    server.await.unwrap();
    Ok(())
}