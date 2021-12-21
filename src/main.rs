use hyper::{Body, Request, Response, Server, StatusCode};
// Import the routerify prelude traits.
extern crate serde_bytes;
extern crate serde_derive;
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr};
mod torrent_process;

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

#[derive(Serialize, Deserialize, Debug)]
struct BodyReq {
    magnet_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BodyResp {
    error: String,
}

// A handler for "/" page.
async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    let state = req.data::<State>().unwrap();
    println!("State value: {}", state.0);

    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/users/:userId" page.
async fn user_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", user_id))))
}

async fn get_torrent_file_to_process_using_magnetic_link(
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let full_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let s: BodyReq = serde_json::from_slice(&full_body).unwrap();
    // todo code need to imple
    Ok(Response::new(Body::from(format!(
        "{}",
        serde_json::json!(s)
    ))))
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}

async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .data(State(100))
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/users/:userId", user_handler)
        .post(
            "/torrent/upload/magnetic-link",
            get_torrent_file_to_process_using_magnetic_link,
        )
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
