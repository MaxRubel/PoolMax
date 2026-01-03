use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

// Async function to handle incoming requests
async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
  // Return a simple "Hello, Rust HTTP Server!" response
  Ok(Response::new(Body::from("Hello, Rust HTTP Server!")))
}

// The main function, annotated with Tokio's main macro to run the async environment
#[tokio::main]
async fn main() {
  // Define the address to listen on (127.0.0.1:3000)
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  // A service that produces other services for each incoming connection
  let make_svc = make_service_fn(|_conn| async {
    // service_fn converts our async handle_request function into a service
    Ok::<_, Infallible>(service_fn(handle_request))
  });

  // Bind the server to the address and serve the service
  let server = Server::bind(&addr).serve(make_svc);

  println!("Server running at 127.0.0.1");

  // Run the server indefinitely
  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }
}
