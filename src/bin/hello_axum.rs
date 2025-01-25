use axum::{
    Router, 
    routing::get, 
    extract::Request as AxumRequest,
};


use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use http::Method;
use std::convert::Infallible;
use bootust::{server::Server as _,
    server::axum::AxumServer};

#[tokio::main]
async fn main() {
    let mut server = AxumServer::new();
    fn handler(req: AxumRequest) -> Result<Response<Body>, Infallible> {
                Ok(Response::new(Body::from("request received\n")))
            }
    

            server.route("/get", http::Method::GET, handler);
            server.route("/post", http::Method::POST, handler);
    
    

    server.run("127.0.0.1:3000").await.unwrap();
}

// #[tokio::main]
async fn hello_axum(){
let router = Router::new().route("/", get(|| async { "Hello, World!" }));

let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
axum::serve(listener, router).await.unwrap();
}
