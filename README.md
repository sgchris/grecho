# 🔄 Echo Server

A high-performance, asynchronous HTTP echo server **built with Rust** and Actix Web. This server mirrors incoming HTTP requests back as responses, making it perfect for testing, debugging, and development purposes.

## 🚀 Features

- **Universal Request Handling**: Accepts all HTTP methods (GET, POST, PUT, DELETE, etc.)
- **Path & Query String Support**: Handles any URI path and query parameters
- **Header Mirroring**: Echoes request headers back in the response (excluding reserved headers)
- **Body Echoing**: Returns the request body as the response body
- **Custom Response Control**: Override response status code and body using special headers
- **High Performance**: Built with Actix Web for maximum speed and efficiency
- **Async Architecture**: Fully asynchronous with automatic worker scaling
- **CLI Interface**: Easy-to-use command-line interface with validation
- **Single File**: Entire implementation in one Rust file for simplicity

## 📋 What is an Echo Server?

An echo server is a network service that sends back the data it receives from clients. It's commonly used for:

- **API Testing**: Test how your application handles different HTTP responses
- **Development**: Mock external services during development
- **Debugging**: Inspect exactly what your client is sending
- **Load Testing**: Simple endpoint for performance testing
- **Network Diagnostics**: Verify connectivity and data transmission

## 🛠️ Installation

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable version)
- Cargo (comes with Rust)

### Setup

1. **Clone or create the project**:
   ```bash
   git clone https://github.com/sgchris/grecho.git
   cd grecho
   cargo build --release
   ```

## 🎯 Usage

### Basic Usage

Start the server with default settings (127.0.0.1:3000):
```bash
cargo run
```

### Custom Configuration

**Specify hostname and port**:
```bash
cargo run -- --hostname 0.0.0.0 --port 8080
```

**Using short flags**:
```bash
cargo run -- -h 192.168.1.100 -p 9000
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--hostname` | `-h` | IP address to bind to | `127.0.0.1` |
| `--port` | `-p` | Port number to bind to | `3000` |
| `--verbose` | `-v` | Display requests and responses details | false |

## 📖 Examples

### Example 1: Simple Echo

**Request**:
```http
POST /api/users HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json
X-Custom-Header: my-value

{"id": 123, "name": "John Doe"}
```

**Response**:
```http
HTTP/1.1 200 OK
Content-Type: application/json
X-Custom-Header: my-value

{"id": 123, "name": "John Doe"}
```

### Example 2: Custom Status Code

**Request**:
```http
GET /test HTTP/1.1
Host: 127.0.0.1:3000
internal.status-code: 404
X-Test-Header: test-value

Request body content
```

**Response**:
```http
HTTP/1.1 404 Not Found
X-Test-Header: test-value

Request body content
```

### Example 3: Custom Response Body

**Request**:
```http
PUT /data HTTP/1.1
Host: 127.0.0.1:3000
internal.response-body: Custom response message
Authorization: Bearer token123

Original request body
```

**Response**:
```http
HTTP/1.1 200 OK

Custom response message
```

## ⚙️ Special Headers

The server recognizes special internal headers for response control:

- **`internal.status-code`**: Override the HTTP response status code
  - Example: `internal.status-code: 503` → Returns HTTP 503
- **`internal.response-body`**: Override the response body content
  - Example: `internal.response-body: Error occurred` → Returns "Error occurred"

## 🧪 Testing with curl

**Basic test**:
```bash
curl -X POST http://127.0.0.1:3000/test \
  -H "Content-Type: application/json" \
  -H "X-Custom: value" \
  -d '{"test": true}'
```

**Test custom status code**:
```bash
curl -X GET http://127.0.0.1:3000/error \
  -H "internal.status-code: 500" \
  -d "Error simulation"
```

**Test custom response body**:
```bash
curl -X POST http://127.0.0.1:3000/custom \
  -H "internal.response-body: Hello World!" \
  -d "Original body"
```

## 🏃‍♂️ Performance

This server is built for high performance:

- **Async Architecture**: Non-blocking I/O operations
- **Worker Pool**: Automatically scales workers to match CPU cores
- **Zero-Copy**: Efficient memory usage with minimal allocations
- **Actix Web**: One of the fastest web frameworks available

## 🔧 Development

### Running Tests
```bash
cargo test
```

### Development Mode (with auto-reload)
```bash
cargo install cargo-watch
cargo watch -x run
```

### Build for Production
```bash
cargo build --release
./target/release/rust-echo-server --hostname 0.0.0.0 --port 8080
```

## 📝 Use Cases

- **API Development**: Mock external services during development
- **Testing**: Verify client request formatting and headers
- **Debugging**: Inspect exactly what your application sends
- **Load Testing**: Simple endpoint for performance benchmarks
- **Integration Testing**: Predictable responses for automated tests
- **Network Diagnostics**: Verify connectivity and data transmission
- **Webhook Testing**: Test webhook payloads and headers

## 🤝 Contributing

Feel free to submit issues, feature requests, or pull requests to improve this echo server!

## 📄 License

This project is open source and available under the [MIT License](LICENSE).

---

**Happy echoing!** 🎉
