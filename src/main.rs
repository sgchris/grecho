use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpServer, Result as ActixResult,
    middleware::Logger,
};
use clap::{Arg, Command};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

// Reserved headers that should not be copied to the response
const RESERVED_HEADERS: &[&str] = &[
    "content-length",
    "user-agent",
    "host",
    "connection",
    "accept",
    "accept-encoding",
    "accept-language",
    "cache-control",
    "upgrade-insecure-requests",
    "sec-fetch-dest",
    "sec-fetch-mode",
    "sec-fetch-site",
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-platform",
    "authorization",
    "cookie",
    "referer",
    "origin",
    "x-forwarded-for",
    "x-forwarded-proto",
    "x-real-ip",
    "transfer-encoding",
    "te",
    "trailer",
    "proxy-authorization",
    "proxy-authenticate",
    "www-authenticate",
];

// Internal headers for controlling response
const INTERNAL_STATUS_CODE_HEADER: &str = "internal.status-code";
const INTERNAL_RESPONSE_BODY_HEADER: &str = "internal.response-body";

async fn echo_handler(req: HttpRequest, body: web::Bytes) -> ActixResult<HttpResponse> {
    let headers = req.headers();
    let reserved_headers: HashSet<&str> = RESERVED_HEADERS.iter().cloned().collect();

    // Check for internal status code override
    let status_code = headers
        .get(INTERNAL_STATUS_CODE_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(200);

    // Check for internal response body override
    let response_body = headers
        .get(INTERNAL_RESPONSE_BODY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| String::from_utf8_lossy(&body).to_string());

    // Create response with the determined status code
    let mut response = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code)
            .unwrap_or(actix_web::http::StatusCode::OK)
    );

    // Copy non-reserved headers to response, excluding internal headers
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();

        // Skip reserved headers and internal control headers
        if !reserved_headers.contains(header_name.as_str())
            && header_name != INTERNAL_STATUS_CODE_HEADER.to_lowercase()
            && header_name != INTERNAL_RESPONSE_BODY_HEADER.to_lowercase() {

            if let Ok(header_value) = value.to_str() {
                response.insert_header((name.clone(), header_value));
            }
        }
    }

    Ok(response.body(response_body))
}

fn validate_hostname(hostname: &str) -> Result<IpAddr, String> {
    IpAddr::from_str(hostname)
        .map_err(|_| format!("Invalid hostname '{}'. Must be a valid IP address.", hostname))
}

fn validate_port(port_str: &str) -> Result<u16, String> {
    let port: u16 = port_str.parse()
        .map_err(|_| format!("Invalid port '{}'. Must be a number between 1 and 65535.", port_str))?;

    if port == 0 {
        return Err("Port cannot be 0. Must be between 1 and 65535.".to_string());
    }

    Ok(port)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse command line arguments
    let matches = Command::new("Echo Server")
        .version("1.0")
        .about("A high-performance echo server that mirrors requests back as responses")
        .arg(
            Arg::new("hostname")
                .short('n')
                .long("hostname")
                .value_name("HOSTNAME")
                .help("The hostname/IP address to bind to")
                .default_value("127.0.0.1")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("The port number to bind to")
                .default_value("3000")
        )
        .get_matches();

    // Extract and validate hostname
    let hostname_str = matches.get_one::<String>("hostname").unwrap();
    let hostname = match validate_hostname(hostname_str) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Extract and validate port
    let port_str = matches.get_one::<String>("port").unwrap();
    let port = match validate_port(port_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let bind_address = SocketAddr::new(hostname, port);

    println!("🚀 Starting Echo Server on http://{}", bind_address);
    println!("📋 Headers that are relevant for the request only, like 'host' or 'user-agent' won't be echoed.");
    println!("⚙️  Use '{}' header to override response status code", INTERNAL_STATUS_CODE_HEADER);
    println!("📝 Use '{}' header to override response body", INTERNAL_RESPONSE_BODY_HEADER);

    // Create and run the HTTP server
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/{path:.*}", web::to(echo_handler))
            .default_service(web::to(echo_handler))
    })
        .bind(&bind_address)?
        .workers(num_cpus::get())
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("127.0.0.1").is_ok());
        assert!(validate_hostname("0.0.0.0").is_ok());
        assert!(validate_hostname("192.168.1.1").is_ok());
        assert!(validate_hostname("::1").is_ok());
        assert!(validate_hostname("invalid-hostname").is_err());
        assert!(validate_hostname("999.999.999.999").is_err());
    }

    #[test]
    fn test_validate_port() {
        assert_eq!(validate_port("3000").unwrap(), 3000);
        assert_eq!(validate_port("8080").unwrap(), 8080);
        assert_eq!(validate_port("65535").unwrap(), 65535);
        assert!(validate_port("0").is_err());
        assert!(validate_port("65536").is_err());
        assert!(validate_port("invalid").is_err());
        assert!(validate_port("-1").is_err());
    }
}
