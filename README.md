# MCP Documentation Server

A Rust-based documentation server built with Tauri 2.0, Axum, jsonrpsee, and Tantivy search.

## Project Structure

This project implements a Tauri 2.0 application with an embedded Axum server that provides jsonrpsee RPC endpoints and Tantivy full-text search capabilities.

- `src-tauri/`: Contains the Rust backend code
  - `src/`: Source code for the Tauri application and Axum server
    - `main.rs`: Entry point for the Tauri application
    - `lib.rs`: Main application logic, including Axum server setup
    - `rpc.rs`: Implementation of jsonrpsee RPC methods
    - `search.rs`: Tantivy search integration
  - `assets/`: Static frontend assets (HTML, CSS, JS)

## Features

- Tauri 2.0 desktop application shell
- Embedded Axum web server
- jsonrpsee RPC service with:
  - Simple ping method for testing
  - Document addition for search indexing
  - Full-text search capability
- Tantivy search engine integration
- Basic HTML/JS frontend to test all functionality

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

1. The Tauri window will display a simple UI with three sections:
   - Ping test: Tests basic RPC connectivity
   - Add Document: Add documents to the search index
   - Search Documents: Search for documents using the Tantivy index

2. You can also test the RPC service directly with curl:

#### Test Ping

```bash
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ping","params":{"message":"Hello MCP"},"id":1}' http://127.0.0.1:3000/rpc
```

#### Add Document

```bash
curl -X POST -H "Content-Type: application/json" -d '{
    "jsonrpc":"2.0",
    "method":"addDocument",
    "params":{
        "document": {
            "id": "doc1",
            "title": "Rust Programming",
            "body": "Rust is a systems programming language focused on safety and speed.",
            "source": "rust-lang.org",
            "version": "1.70"
        }
    },
    "id":2
}' http://127.0.0.1:3000/rpc
```

#### Search Documents

```bash
curl -X POST -H "Content-Type: application/json" -d '{
    "jsonrpc":"2.0",
    "method":"searchDocuments",
    "params":{
        "query": "Rust safety",
        "limit": 5
    },
    "id":3
}' http://127.0.0.1:3000/rpc
```

## Next Steps

1. Add SQLite database integration for persistent storage of document metadata
2. Develop a more comprehensive frontend with SolidJS
3. Implement user authentication and authorization
4. Add support for multiple document formats and sources
