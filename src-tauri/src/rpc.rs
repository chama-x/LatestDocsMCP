use crate::AppState;
use crate::search::SearchableDocument;
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::routing::options;
use axum::Router as AxumRouter;
use tokio::sync::Mutex;
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct PingParams {
    #[serde(default = "default_ping_message")]
    pub message: String,
}

// Default function for the message field
fn default_ping_message() -> String {
    "No message provided".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResponse {
    pub reply: String,
}

// New argument/response types for search
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

/// Define the RPC trait
#[rpc(server)]
pub trait McpRpc {
    #[method(name = "ping")]
    async fn ping(&self, params: PingParams) -> Result<PingResponse, jsonrpsee::types::error::ErrorObject<'static>>;

    #[method(name = "addDocument")]
    async fn add_document(&self, params: AddDocumentParams) -> Result<String, jsonrpsee::types::error::ErrorObject<'static>>;
    
    #[method(name = "searchDocuments")]
    async fn search_documents(&self, params: SearchParams) -> Result<SearchResponse, jsonrpsee::types::error::ErrorObject<'static>>;
}

/// Implement the RPC server logic
#[derive(Clone)]
pub struct McpRpcServerImpl {
    app_state: Arc<AppState>, // Store AppState
}

impl McpRpcServerImpl {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl McpRpcServer for McpRpcServerImpl {
    async fn ping(&self, params: PingParams) -> Result<PingResponse, jsonrpsee::types::error::ErrorObject<'static>> {
        println!("Received ping with message: {}", params.message);
        Ok(PingResponse {
            reply: format!("pong - received: {}", params.message),
        })
    }

    async fn add_document(&self, params: AddDocumentParams) -> Result<String, jsonrpsee::types::error::ErrorObject<'static>> {
        println!("RPC: add_document called with id: {}", params.document.id);
        // Writer memory budget: 50MB per add operation, adjust as needed
        const WRITER_MEMORY_BUDGET: usize = 50_000_000; 
        match self.app_state.search_service.add_document(params.document.clone(), WRITER_MEMORY_BUDGET) {
            Ok(_) => Ok(format!("Document {} added successfully.", params.document.id)),
            Err(e) => {
                eprintln!("Failed to add document: {:?}", e);
                Err(jsonrpsee::types::error::ErrorObject::owned(
                    -32603, // Internal error code
                    format!("Failed to add document: {}", e),
                    None::<()>
                ))
            }
        }
    }

    async fn search_documents(&self, params: SearchParams) -> Result<SearchResponse, jsonrpsee::types::error::ErrorObject<'static>> {
        println!("RPC: search_documents called with query: {}", params.query);
        let limit = params.limit.unwrap_or(10); // Default limit
        match self.app_state.search_service.search_documents(&params.query, limit) {
            Ok(documents) => Ok(SearchResponse { documents }),
            Err(e) => {
                eprintln!("Failed to search documents: {:?}", e);
                Err(jsonrpsee::types::error::ErrorObject::owned(
                    -32603, // Internal error code
                    format!("Failed to search documents: {}", e),
                    None::<()>
                ))
            }
        }
    }
}

// Create a wrapper around the RPC module to make it shareable across threads
struct SharedRpcModule {
    rpc_impl: Arc<McpRpcServerImpl>, // Store the implementation directly
}

impl SharedRpcModule {
    fn new(app_state: Arc<AppState>) -> Self {
        let rpc_impl = Arc::new(McpRpcServerImpl::new(app_state));
        Self { 
            rpc_impl,
        }
    }
}

impl Clone for SharedRpcModule {
    fn clone(&self) -> Self {
        Self {
            rpc_impl: Arc::clone(&self.rpc_impl),
        }
    }
}

// Helper function to manually process RPC requests
async fn process_rpc_request(rpc_impl: &McpRpcServerImpl, request_str: &str) -> String {
    // Parse the JSON-RPC request
    let request: serde_json::Value = match serde_json::from_str(request_str) {
        Ok(req) => req,
        Err(e) => {
            return format!(
                r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":-32700,"message":"Parse error","data":"{}"}}}}"#,
                e
            );
        }
    };
    
    // Extract the method, params, and id
    let method = match request.get("method") {
        Some(serde_json::Value::String(m)) => m.as_str(),
        _ => {
            return r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"Invalid Request","data":"Missing method"}}"#.to_string();
        }
    };
    
    let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
    
    // Handle each method separately
    match method {
        "ping" => {
            let params = match request.get("params") {
                Some(p) => match serde_json::from_value::<PingParams>(p.clone()) {
                    Ok(params) => params,
                    Err(e) => {
                        return format!(
                            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"Invalid params","data":"{}"}}}}"#,
                            id, e
                        );
                    }
                },
                None => PingParams { message: default_ping_message() },
            };
            
            match rpc_impl.ping(params).await {
                Ok(response) => {
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"result":{{"reply":"{}"}}}}"#,
                        id, response.reply
                    )
                }
                Err(e) => {
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":{},"message":"{}","data":null}}}}"#,
                        id, e.code(), e.message()
                    )
                }
            }
        },
        "addDocument" => {
            let params = match request.get("params") {
                Some(p) => match serde_json::from_value::<AddDocumentParams>(p.clone()) {
                    Ok(params) => params,
                    Err(e) => {
                        return format!(
                            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"Invalid params","data":"{}"}}}}"#,
                            id, e
                        );
                    }
                },
                None => {
                    return format!(
                        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"Invalid params","data":"Missing params for addDocument"}}}}"#,
                        id
                    );
                }
            };
            
            match rpc_impl.add_document(params).await {
                Ok(result) => {
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"result":"{}"}}"#,
                        id, result.replace("\"", "\\\"")
                    )
                }
                Err(e) => {
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":{},"message":"{}","data":null}}}}"#,
                        id, e.code(), e.message()
                    )
                }
            }
        },
        "searchDocuments" => {
            let params = match request.get("params") {
                Some(p) => match serde_json::from_value::<SearchParams>(p.clone()) {
                    Ok(params) => params,
                    Err(e) => {
                        return format!(
                            r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"Invalid params","data":"{}"}}}}"#,
                            id, e
                        );
                    }
                },
                None => {
                    return format!(
                        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32602,"message":"Invalid params","data":"Missing params for searchDocuments"}}}}"#,
                        id
                    );
                }
            };
            
            match rpc_impl.search_documents(params).await {
                Ok(result) => {
                    let documents_json = serde_json::to_string(&result.documents).unwrap_or_else(|_| "[]".to_string());
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"result":{{"documents":{}}}}}"#,
                        id, documents_json
                    )
                }
                Err(e) => {
                    format!(
                        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":{},"message":"{}","data":null}}}}"#,
                        id, e.code(), e.message()
                    )
                }
            }
        },
        _ => {
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32601,"message":"Method not found","data":"{}"}}}}"#,
                id, method
            )
        }
    }
}

// Axum integration
pub fn create_rpc_router(app_state: Arc<AppState>) -> AxumRouter {
    let shared_module = SharedRpcModule::new(app_state);
    
    AxumRouter::new()
        .route("/rpc", axum::routing::post(move |req: axum::http::Request<axum::body::Body>| {
            let shared_module = shared_module.clone();
            async move {
                // Extract the request body
                let bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::BAD_REQUEST)
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                            .header("Access-Control-Allow-Headers", "Content-Type")
                            .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                            .unwrap();
                    }
                };
                
                let request_str = match std::str::from_utf8(&bytes) {
                    Ok(req) => req,
                    Err(err) => {
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::BAD_REQUEST)
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                            .header("Access-Control-Allow-Headers", "Content-Type")
                            .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                            .unwrap();
                    }
                };
                
                // Log the raw request for debugging
                println!("RAW REQUEST RECEIVED: {}", request_str);
                
                // Process the request using our custom handler instead of jsonrpsee
                let rpc_impl = &shared_module.rpc_impl;
                let response_body = process_rpc_request(rpc_impl, request_str).await;
                println!("RESPONSE SENT: {}", response_body);
                
                axum::response::Response::builder()
                    .status(axum::http::StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type")
                    .body(axum::body::Body::from(response_body))
                    .unwrap()
            }
        }))
        // Handle OPTIONS requests for CORS preflight
        .route("/rpc", options(|| async { 
            axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(axum::body::Body::empty())
                .unwrap()
        }))
} 