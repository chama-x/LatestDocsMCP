# MCP Documentation Server

A Rust-based documentation server built with Tauri 2.0, Axum, and jsonrpsee.

## Project Structure

This project implements a Tauri 2.0 application with an embedded Axum server that provides jsonrpsee RPC endpoints.

- `src-tauri/`: Contains the Rust backend code
  - `src/`: Source code for the Tauri application and Axum server
    - `main.rs`: Entry point for the Tauri application
    - `lib.rs`: Main application logic, including Axum server setup
    - `rpc.rs`: Implementation of jsonrpsee RPC methods
  - `assets/`: Static frontend assets (HTML, CSS, JS)

## Features

- Tauri 2.0 desktop application shell
- Embedded Axum web server
- jsonrpsee RPC service with a simple ping method
- Basic HTML/JS frontend to test the RPC service

## Development

### Prerequisites

- Rust 1.77+ and Cargo
- Tauri 2.0 CLI (`cargo install tauri-cli --version '^2.0.0-beta'`)
- Node.js and npm (for frontend development)

### Running the App

```bash
# Development mode
cargo tauri dev

# Build for production
cargo tauri build
```

### Testing the RPC Service

Once the application is running:

1. The Tauri window will display a simple UI with a "Test Ping RPC" button
2. Click the button to send a request to the embedded Axum server
3. The response from the jsonrpsee RPC service will be displayed

You can also test the RPC service directly with curl:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ping","params":{"message":"Hello MCP"},"id":1}' http://127.0.0.1:3000/rpc
```

## Next Steps

1. Implement additional RPC methods for documentation functionality
2. Integrate Tantivy for full-text search
3. Develop a more comprehensive frontend
4. Add database support for storing documentation metadata 