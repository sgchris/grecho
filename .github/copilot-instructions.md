# GitHub Copilot Instructions

## Project Overview

This is a Rust-based project focused on building a high-performance async echo web server. The server supports all kinds of HTTP requests, including GET, POST, PATCH, and DELETE. It is designed to handle a large number of concurrent connections efficiently.

## Coding Style
- Use `snake_case` for variables and functions.
- Prefer `match` over `if let` when handling enums.
- Avoid `unwrap()` in production code when you are not sure that it will not fail.
- Use the most common Rust idioms and patterns.

## Libraries and Tools
- actix-web for the web server
- clap for command-line argument parsing
- env_logger for logging
- num_cpus for CPU core detection

## Patterns to Follow
- Avoid unused code and imports.
- Use `Result<T, E>` for error handling.
- Implement traits for modularity, if there are any.
- Favor immutability and explicit lifetimes.
- Avoid unnecessary cloning of data.
- Use async/await for asynchronous operations.
- use Settings.toml for configuration, create a `Settings` struct to read the settings from the file. Use `serde` for deserialization.
- after every major change, run `cargo fmt` to format the code, `cargo clippy` to check for common mistakes and improve the code quality, and `cargo build` to ensure the code compiles correctly without errors and warnings.

## Don't Suggest
- Blocking I/O operations
- Unsafe code unless explicitly marked
- Suppressing warnings without justification
