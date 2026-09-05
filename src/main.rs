mod hnu;
mod mcp;

use std::sync::Arc;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use hnu::session::SessionManager;
use mcp::server::HnuDormServer;

#[tokio::main]
async fn main() -> Result<()> {
    // 日志输出到 stderr，不会污染 MCP 的 stdout / JSON-RPC 通道。
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("hnu_query=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    let session = Arc::new(SessionManager::new());

    let server = HnuDormServer::new(session);

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}