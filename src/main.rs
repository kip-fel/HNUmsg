mod hnu;
mod mcp;

use std::sync::Arc;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

use hnu::session::SessionManager;
use mcp::server::HnuDormServer;

#[tokio::main]
async fn main() -> Result<()> {
    let session = Arc::new(SessionManager::new());

    let server = HnuDormServer::new(session);

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}