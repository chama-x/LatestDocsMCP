mod search;

use std::sync::Arc;
use search::SearchService;
use tempfile::tempdir;
use tauri::State;
use serde::{Serialize, Deserialize};
use tauri::Emitter;
use tauri::Manager;
use tauri::Listener;

// Import the SearchableDocument type from the search module
use search::SearchableDocument;

// Shared application state
pub struct AppState {
    pub search_service: Arc<SearchService>,
    // Add more shared resources as needed
}

impl AppState {
    fn new() -> Result<Self, anyhow::Error> {
        // For development, use a temporary directory for the index
        // In production, you'd use a persistent path
        let temp_dir = tempdir()?;
        let index_dir = temp_dir.keep();
        
        println!("Initializing Tantivy index at: {:?}", index_dir);
        
        let search_service = Arc::new(SearchService::new(index_dir)?);
        
        Ok(Self {
            search_service,
        })
    }
}

// Define the types needed for Tauri commands
#[derive(Serialize, Deserialize, Debug)]
pub struct PingParams {
    #[serde(default = "default_ping_message")]
    pub message: String,
}

fn default_ping_message() -> String {
    "No message provided".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResponse {
    pub reply: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddDocumentParams {
    pub document: SearchableDocument,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponse {
    pub documents: Vec<SearchableDocument>,
}

// Tauri commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn ping(params: PingParams) -> Result<PingResponse, String> {
    println!("Received ping with message: {}", params.message);
    Ok(PingResponse {
        reply: format!("pong - received: {}", params.message),
    })
}

#[tauri::command]
async fn add_document(
    state: State<'_, AppState>,
    params: AddDocumentParams
) -> Result<String, String> {
    println!("Command: add_document called with id: {}", params.document.id);
    // Writer memory budget: 50MB per add operation, adjust as needed
    const WRITER_MEMORY_BUDGET: usize = 50_000_000; 
    
    match state.search_service.add_document(params.document.clone(), WRITER_MEMORY_BUDGET) {
        Ok(_) => Ok(format!("Document {} added successfully.", params.document.id)),
        Err(e) => {
            eprintln!("Failed to add document: {:?}", e);
            Err(format!("Failed to add document: {}", e))
        }
    }
}

#[tauri::command]
async fn search_documents(
    state: State<'_, AppState>,
    params: SearchParams
) -> Result<SearchResponse, String> {
    println!("Command: search_documents called with query: {}", params.query);
    let limit = params.limit.unwrap_or(10); // Default limit
    
    match state.search_service.search_documents(&params.query, limit) {
        Ok(documents) => Ok(SearchResponse { documents }),
        Err(e) => {
            eprintln!("Failed to search documents: {:?}", e);
            Err(format!("Failed to search documents: {}", e))
        }
    }
}

#[tauri::command]
async fn emit_event_example(window: tauri::Window) -> Result<(), String> {
    window.emit("custom-event", Some("Event payload"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_background_task(window: tauri::Window) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        for i in 0..10 {
            // Do some work
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            // Update the frontend
            let _ = window.emit("progress", i);
        }
        
        // Notify completion
        let _ = window.emit("task-complete", true);
    });
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state
    let app_state = match AppState::new() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Failed to initialize app state: {}", err);
            return;
        }
    };
    
    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                  tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
                )?;
            }
            
            // Setup event listeners
            let window = app.get_webview_window("main").unwrap();
            window.listen("frontend-event", |event| {
                println!("Got event from frontend: {:?}", event.payload());
            });
            
            Ok(())
        })
        .manage(app_state) // Share state with commands
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            ping, 
            add_document, 
            search_documents,
            emit_event_example,
            run_background_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
