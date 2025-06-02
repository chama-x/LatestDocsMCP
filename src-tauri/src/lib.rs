mod rpc;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// Shared application state
pub struct AppState {
    // Example: could hold a Tantivy index writer, DB connection pool, etc.
    // We'll populate this later.
}

impl AppState {
    fn new() -> Self {
        Self {}
    }
}

async fn run_axum_server(_app_state: Arc<AppState>) {
    // Create the RPC router
    let rpc_router = rpc::create_rpc_router();

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    // Create an Axum router
    let app = axum::Router::new()
        .merge(rpc_router)
        // Add health check endpoint
        .route("/health", axum::routing::get(|| async { "OK" }))
        // Add CORS middleware
        .layer(cors);

    // Define the address for the Axum server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);
    println!("Axum server with RPC endpoint starting on http://{}", addr);

    // Run the Axum server
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("Listening on {}", listener.local_addr().unwrap());
            if let Err(err) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", err);
            }
        }
        Err(err) => {
            eprintln!("Failed to bind to {}: {}", addr, err);
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state
    let app_state = Arc::new(AppState::new());
    let app_state_clone = Arc::clone(&app_state);

    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                  tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
                )?;
            }
            
            // Use the Tauri runtime to spawn the server task
            tauri::async_runtime::spawn(async move {
                run_axum_server(app_state_clone).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
