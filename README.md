# Grecho
Grecho is a simple asynchronous HTTP echo server written in Rust. It returns the request body, headers, and URI in the response, with support for customizing the response via internal headers.

## Features
**Echo Request Body**: Returns the same body as the request.
**Echo Request Headers**: Returns the same headers as the request (excluding internal headers and the host header).

### Customizable Response:
- Use **internal.status-code** to set a custom HTTP status code.
- Use **internal.response-body** to set a custom response body.

## Installation
### Prerequisites
Rust (version 1.60 or higher)<br>
Cargo (Rust's package manager)<br>

### Steps
#### Clone the repository

git clone https://github.com/your-username/grecho.git<br>
cd grecho<Br>

#### Build the project
cargo build --release

#### Run the server
By default, the server binds to 127.0.0.1:3000. You can specify a custom host and port:<Br>
```
cargo run -- --host 127.0.0.1 --port 3000<Br>
cargo run -- -H 127.0.0.1 -p 3000<br>
```

## Send Requests

curl -X POST http://127.0.0.1:3000/example -d "Hello, World!" -H "Content-Type: text/plain"<Br>
curl -X POST http://127.0.0.1:3000/example -d "Hello, World!" -H "internal.status-code: 202"<Br>
curl -X POST http://127.0.0.1:3000/example -d "Hello, World!" -H "internal.response-body: Custom Body"<Br>

## Configuration
Command-Line Arguments
```
-H, --host	Host to bind the server to	127.0.0.1
-p, --port	Port to bind the server to	3000
```

### Internal Headers
| Header Name |	Description |	Default Value |
|-------------| ---------- | ------------- |
| internal.status-code |	Set the HTTP status code |	200 |
| internal.response-body |	Set the response body |	Request body |

