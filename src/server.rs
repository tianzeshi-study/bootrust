pub mod axum;

use async_trait::async_trait;
use http::{Request, Response, StatusCode};

#[async_trait]
pub trait Server{
    type Request;
    type Response;
    type Error;
    // type Middleware;
    type Context;

    // 路由
    fn route(&mut self, path: &str, method: http::Method, handler: fn(Self::Context) -> Result<Self::Response, Self::Error>);

    // 添加中间件
    // fn add_middleware(&mut self, middleware: Self::Middleware);

    // 处理请求
    async fn handle_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;

    // 运行服务器
    async fn run(&self, addr: &str) -> Result<(), Self::Error>;

    // fn service<S>(&mut self, path: &str, service: S)
    // where
    // S: tower::Service<Self::Request> +Clone + Send + Sync +'static;
    // s::Response = Self::Response, 
    // s::Error = Self::Error,
        // S::Future: Send;


}