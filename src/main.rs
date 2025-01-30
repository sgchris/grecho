use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use clap::Parser; // For command-line argument parsing

// List of headers that should not be echoed in the response
const EXCLUDED_HEADERS: &[&str] = &[
    "accept",
    "user-agent",

    // Content-related
    "content-length",
    "content-type",
    "content-encoding",
    "content-disposition",
    "content-range",
    "content-language",
    // Transfer-related
    "transfer-encoding",
    "te",
    // Connection-related
    "connection",
    "keep-alive",
    "upgrade",
    "proxy-connection",
    // Authentication/Authorization
    "authorization",
    "proxy-authenticate",
    "proxy-authorization",
    // Request-specific
    "host",
    "expect",
    "max-forwards",
    "if-match",
    "if-none-match",
    "if-modified-since",
    "if-unmodified-since",
    "if-range",
];

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Host to bind the server to
    #[clap(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[clap(short, long, default_value = "3000")]
    port: u16,
}


#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Construct the server address
    let addr = SocketAddr::new(args.host.parse().expect("Invalid host"), args.port);

    println!("Starting server on {}:{}", args.host, args.port);

    // Create TCP listener
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (tcp_stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(tcp_stream);

        // Spawn a task to serve the connection
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}

async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!();
    println!("New request");
    // Extract request components
    let uri = req.uri().to_string();
    let headers = req.headers().clone();
    let body_bytes = req.collect().await.unwrap().to_bytes();

    // Initialize response builder
    let mut response_builder = Response::builder();

    // Get status code from internal headers or default to 200
    let status_code = headers
        .get("internal.status-code")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u16>().ok())
        .map(hyper::StatusCode::from_u16)
        .unwrap_or(Ok(hyper::StatusCode::OK))
        .unwrap_or(hyper::StatusCode::OK);

    // Set status code
    println!("Response status code: {}", status_code);
    response_builder = response_builder.status(status_code);

    // Set request-uri header
    println!("Response uri: {}", uri);
    response_builder = response_builder.header("request-uri", uri);

    // Copy non-internal and non-excluded headers
    let mut headers_string = String::new();
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !key_str.starts_with("internal.") && !EXCLUDED_HEADERS.contains(&key_str.as_str()) {
            headers_string.push_str(&format!("\t{}: {};\r\n", key, value.to_str().unwrap()));
            response_builder = response_builder.header(key, value);
        }
    }

    // Build the string with headers
    /*
    let response_headers_for_output = response_builder.headers().unwrap();
    let mut headers_string = String::new();
    for (key, value) in response_headers_for_output {
        headers_string.push_str(&format!("{}: {}; ", key, value.to_str().unwrap()));
    }
    */
    println!("Response headers: \n{}", headers_string);

    // Get response body from internal headers or use request body
    let response_body = if let Some(internal_body) = headers.get("internal.response-body") {
        // Try to convert the header value to a string
        if let Ok(body_str) = internal_body.to_str() {
            // Use the internal response body
            Bytes::from(body_str.as_bytes().to_vec())
        } else {
            // If we can't convert the header value to string, fallback to request body
            body_bytes
        }
    } else {
        // If there's no internal response body header, use the request body
        body_bytes
    };
    println!("Response body: {:?}", response_body.clone());

    // Build and return response
    Ok(response_builder.body(Full::new(response_body)).unwrap())
}

