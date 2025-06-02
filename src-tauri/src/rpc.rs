use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::ServerHandle,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::routing::{post, options};
use axum::Router as AxumRouter;

#[derive(Serialize, Deserialize, Debug)]
pub struct PingParams {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResponse {
    pub reply: String,
}

/// Define the RPC trait
#[rpc(server)]
pub trait McpRpc {
    #[method(name = "ping")]
    async fn ping(&self, params: PingParams) -> Result<PingResponse, jsonrpsee::types::error::ErrorObject<'static>>;
}

/// Implement the RPC server logic
pub struct McpRpcServerImpl;

impl McpRpcServerImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpRpcServer for McpRpcServerImpl {
    async fn ping(&self, params: PingParams) -> Result<PingResponse, jsonrpsee::types::error::ErrorObject<'static>> {
        tracing::info!("Received ping with message: {}", params.message);
        Ok(PingResponse {
            reply: format!("pong - received: {}", params.message),
        })
    }
}

pub async fn run_rpc_server(addr: SocketAddr) -> Result<(SocketAddr, ServerHandle), anyhow::Error> {
    tracing::info!("Starting RPC server on {}...", addr);

    let server = jsonrpsee::server::Server::builder()
        .build(addr)
        .await?;

    let local_addr = server.local_addr()?;
    
    let module = McpRpcServerImpl::new().into_rpc();
    let handle = server.start(module);
    
    tracing::info!("RPC server started on {}", local_addr);

    Ok((local_addr, handle))
}

// Axum integration - using direct JSON handling for simplicity
pub fn create_rpc_router() -> AxumRouter {
    // Create a simple handler that processes JSON-RPC requests manually
    AxumRouter::new()
        .route("/rpc", post(handle_rpc_request))
        // Handle OPTIONS requests for CORS preflight
        .route("/rpc", options(|| async { 
            axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .body(axum::body::Body::empty())
                .unwrap()
        }))
}

async fn handle_rpc_request(
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    // Extract the request body
    let bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                .unwrap();
        }
    };
    
    let request_str = match std::str::from_utf8(&bytes) {
        Ok(req) => req,
        Err(err) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                .unwrap();
        }
    };
    
    // Parse the JSON-RPC request
    let request: serde_json::Value = match serde_json::from_str(request_str) {
        Ok(req) => req,
        Err(err) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                .unwrap();
        }
    };
    
    // Extract method and params
    let method = match request.get("method") {
        Some(serde_json::Value::String(method)) => method.as_str(),
        _ => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from("{\"error\": \"Invalid method\"}"))
                .unwrap();
        }
    };
    
    let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
    
    // Handle the ping method
    if method == "ping" {
        if let Some(params) = request.get("params") {
            if let Ok(ping_params) = serde_json::from_value::<PingParams>(params.clone()) {
                let rpc_impl = McpRpcServerImpl::new();
                match rpc_impl.ping(ping_params).await {
                    Ok(response) => {
                        let json_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": response,
                            "id": id
                        });
                        
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(axum::body::Body::from(json_response.to_string()))
                            .unwrap();
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": err.code(),
                                "message": err.message()
                            },
                            "id": id
                        });
                        
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(axum::body::Body::from(json_response.to_string()))
                            .unwrap();
                    }
                }
            }
        }
    }
    
    // Method not found
    let json_response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32601,
            "message": "Method not found"
        },
        "id": id
    });
    
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(json_response.to_string()))
        .unwrap()
} 