use crate::server::Server;
use async_trait::async_trait;
use tokio;
use axum::{
    body::Body,
    extract::Request as AxumRequest,
    http::{self, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post, Route},
    Router,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use tower_service::Service;

#[derive(Debug)]
pub struct AxumServer {
    router: Router,
}

impl AxumServer {
    pub fn new() -> Self {
        AxumServer {
            router: Router::new(),
        }
    }
}

#[async_trait]
impl Server for AxumServer {
    // type Request = AxumRequest;
    type Request = Request<Body>;
    type Response = Response<Body>;
    type Error = Infallible;
    // type Middleware =
        // fn(AxumRequest, Next<Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response<Body>> + Send>>;
    type Context = AxumRequest;

    fn route(
        &mut self,
        path: &str,
        method: http::Method,
        handler: fn(Self::Context) -> Result<Self::Response, Self::Error>,
    ) {
        async fn handler_wrapper<F, T, E>(
            req: AxumRequest,
            handler: F,
        ) -> Result<Response<Body>, Infallible>
        where
            F: Fn(AxumRequest) -> Result<T, E> + Send + Sync + 'static,
            T: IntoResponse,
            E: IntoResponse,
        {
            let result = handler(req);
            match result {
                Ok(res) => Ok(res.into_response()),
                Err(err) => Ok(err.into_response()),
            }
        }

        let route = match method {
            http::Method::GET => get(move |req| handler_wrapper(req, handler)),
            http::Method::POST => post(move |req| handler_wrapper(req, handler)),
            // ... 可以根据需要添加其他 HTTP 方法
            _ => get(move |req| handler_wrapper(req, handler)), // 默认使用 get
        };
        self.router = self.router.clone().route(path, route);
    }
/*
    fn add_middleware(&mut self, middleware: Self::Middleware) {
        async fn middleware_wrapper(
            req: AxumRequest,
            next: Next<Body>,
        ) -> impl IntoResponse {
            middleware(req, next).await
        }
        self.router = self.router.layer(middleware::from_fn(middleware_wrapper));
    }
        fn add_middleware(&mut self, middleware: Self::Middleware) {
            let middleware_wrapper = |req: AxumRequest, next: Next<Body>| async move {
                middleware(req, next).await
            };
            self.router = self.router.layer(middleware::from_fn(middleware_wrapper));
        }
 */   

    async fn handle_request(&self, _request: Self::Request) -> Result<Self::Response, Self::Error> {
        // Axum 会自动处理请求，这里不需要做任何事情
        unreachable!()
    }

    async fn run(&self, addr: &str) -> Result<(), Self::Error> {
        let addr = addr.parse::<SocketAddr>().unwrap();
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, self.router.clone()).await.unwrap();
        Ok(())
    }

    /*
    fn service<S>(&mut self, path: &str, service: S)
    where
    S: tower::Service<Self::Request> + Clone +Send + Sync +'static,
    //  s::Response = Self::Response, 
    //  s::Error = Self::Error,
                // S::Future: Send,
    {
        let router = self.router.clone();
        self.router = router.nest(
            path,
            Router::new().fallback(move |req| async move {
                let mut service = service.clone();
                service.call(req).await
            })
        );
    }
*/

}



#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use reqwest;
    use std::net::{SocketAddr, TcpListener};

   
    
 
    #[tokio::test]
    async fn test_get_route() {
        // 创建一个 AxumServer 实例
        let mut server = AxumServer::new();

        // 定义一个简单的 GET 路由处理函数
        fn get_handler(req: AxumRequest) -> Result<Response<Body>, Infallible> {
            Ok(Response::new(Body::from("GET request received")))
        }

        // 注册路由
        server.route("/", http::Method::GET, get_handler);

        // 启动服务器
        let addr = run_server_in_background(server).await;
        println!("server running ");

        // 发送 GET 请求
        let client = reqwest::Client::new();
        let response = client.get(&format!("http://{}/", addr)).send().await.unwrap();

        // 检查响应状态码
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        // 检查响应体
        let body = response.text().await.unwrap();
        assert_eq!(body, "GET request received");
    }

    // #[tokio::test]
    async fn test_post_route() {
        // 创建一个 AxumServer 实例
        let mut server = AxumServer::new();

        // 定义一个简单的 POST 路由处理函数
        fn post_handler(req: AxumRequest) -> Result<Response<Body>, Infallible> {
            Ok(Response::new(Body::from("POST request received")))
        }

        // 注册路由
        server.route("/", http::Method::POST, post_handler);

        // 启动服务器
        let addr = run_server_in_background(server).await;

        // 发送 POST 请求
        let client = reqwest::Client::new();
        let response = client.post(&format!("http://{}/", addr)).send().await.unwrap();

        // 检查响应状态码
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        // 检查响应体
        let body = response.text().await.unwrap();
        assert_eq!(body, "POST request received");
    }

    // 辅助函数：在后台运行服务器
    async fn run_server_in_background(server: AxumServer) -> SocketAddr {
        // 找到一个可用的端口
        let listener = TcpListener::bind("127.0.0.1:4399").unwrap();
        let addr = listener.local_addr().unwrap();

        // 在后台运行服务器
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, server.router.clone()).await.unwrap();
        });

        addr
    }
   
}