mod rpc;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use log;

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

async fn run_axum_server(app_state: Arc<AppState>) {
    // Create the RPC router and module
    let (rpc_router, rpc_module) = rpc::create_rpc_router();

    // Create an Axum router
    let app = axum::Router::new()
        .merge(rpc_router)
        // Add health check endpoint
        .route("/health", axum::routing::get(|| async { "OK" }))
        .with_state(rpc_module);

    // Define the address for the Axum server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);
    tracing::info!("Axum server with RPC endpoint starting on http://{}", addr);

    // Run the Axum server
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to start Axum server");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("info".parse().unwrap())
            .add_directive("mcp_docs_server_core=debug".parse().unwrap()))
        .init();

    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                  tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
                )?;
            }
            
            let app_state_clone = Arc::clone(&app_state);

            // Spawn the Axum server in a separate Tokio task
            tokio::spawn(async move {
                run_axum_server(app_state_clone).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
