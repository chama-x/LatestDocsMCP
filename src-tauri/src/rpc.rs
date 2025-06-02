use jsonrpsee::{
    core::{async_trait, Error as JsonRpcError},
    proc_macros::rpc,
    server::ServerHandle,
    RpcModule,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::routing::post;
use axum::Router as AxumRouter;

#[derive(Serialize, Deserialize, Debug)]
pub struct PingParams {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponse {
    pub reply: String,
}

/// Define the RPC trait
#[rpc(server)]
pub trait McpRpc {
    #[method(name = "ping")]
    async fn ping(&self, params: PingParams) -> Result<PingResponse, JsonRpcError>;
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
    async fn ping(&self, params: PingParams) -> Result<PingResponse, JsonRpcError> {
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

// Axum integration
pub fn create_rpc_router() -> AxumRouter {
    let rpc_impl = McpRpcServerImpl::new();
    let module = rpc_impl.into_rpc();
    
    AxumRouter::new().route("/rpc", post(
        move |req: axum::http::Request<axum::body::Body>| {
            let module = module.clone();
            async move {
                let bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await.unwrap();
                let request = std::str::from_utf8(&bytes).unwrap();
                
                let response = module.raw_json_request(request).await;
                
                match response {
                    Ok(result) => axum::response::Response::builder()
                        .status(axum::http::StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(axum::body::Body::from(result))
                        .unwrap(),
                    Err(err) => axum::response::Response::builder()
                        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from(format!("{{\"error\": \"{}\"}}", err)))
                        .unwrap(),
                }
            }
        }
    ))
} 