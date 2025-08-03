use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpServer, Result as ActixResult,
    middleware::Logger,
};
use clap::{Arg, Command};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
struct Settings {
    host: String,
    port: u16,
}

impl Settings {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let settings_content = std::fs::read_to_string("Settings.toml")?;
        let settings: Settings = toml::from_str(&settings_content)?;
        Ok(settings)
    }
}

async fn echo_handler(req: HttpRequest, body: web::Bytes, verbose: web::Data<bool>) -> ActixResult<HttpResponse> {
    let headers = req.headers();
    let reserved_headers: HashSet<&str> = RESERVED_HEADERS.iter().cloned().collect();

    // Log incoming request if verbose mode is enabled
    if **verbose {
        println!("\n📥 INCOMING REQUEST:");
        println!("   {} {}{}", req.method(), req.path(), req.query_string());
        if !headers.is_empty() {
            println!("   Headers:");
            for (name, value) in headers.iter() {
                if let Ok(value_str) = value.to_str() {
                    println!("     {}: {}", name, value_str);
                }
            }
        } else {
            println!("   No headers");
        }

        if !body.is_empty() {
            println!("   Body: {}", String::from_utf8_lossy(&body));
        }
    }

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

    // Log outgoing response if verbose mode is enabled
    if **verbose {
        println!("\n📤 OUTGOING RESPONSE:");
        println!("   Status: {}", status_code);
        println!("   Headers:");
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            if !reserved_headers.contains(header_name.as_str())
                && header_name != INTERNAL_STATUS_CODE_HEADER.to_lowercase()
                && header_name != INTERNAL_RESPONSE_BODY_HEADER.to_lowercase() {
                if let Ok(header_value) = value.to_str() {
                    println!("     {}: {}", name, header_value);
                }
            }
        }
        println!("   Body: {}", response_body);
        println!();
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

    // Load settings from Settings.toml, with fallback defaults
    let settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("Warning: Could not load Settings.toml ({}). Using default values.", e);
        Settings {
            host: "127.0.0.1".to_string(),
            port: 3001,
        }
    });

    // Parse command line arguments using static defaults, will override with settings if not provided by user
    let matches = Command::new("Echo Server")
        .version("1.0.1")
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
                .default_value("3001")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging of requests and responses")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Extract hostname - use CLI arg if provided, otherwise use settings
    let hostname_str = matches.get_one::<String>("hostname")
        .map(|s| s.as_str())
        .unwrap_or(&settings.host);
    let hostname = match validate_hostname(hostname_str) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Extract port - use CLI arg if provided, otherwise use settings
    let port = if let Some(port_str) = matches.get_one::<String>("port") {
        match validate_port(port_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        settings.port
    };

    // Extract verbose flag
    let verbose = matches.get_flag("verbose");

    let bind_address = SocketAddr::new(hostname, port);

    println!("🚀 Starting Echo Server on http://{}", bind_address);
    println!("⚙️  Configuration loaded from Settings.toml (host: {}, port: {})", settings.host, settings.port);
    println!("📋 Headers that are relevant for the request only, like 'host' or 'user-agent' won't be echoed.");
    println!("⚙️  Use '{}' header to override response status code", INTERNAL_STATUS_CODE_HEADER);
    println!("📝 Use '{}' header to override response body", INTERNAL_RESPONSE_BODY_HEADER);
    if verbose {
        println!("🔍 Verbose mode enabled - requests and responses will be logged");
    }

    // Create and run the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(verbose))
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
