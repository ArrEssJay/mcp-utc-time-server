// MCP UTC Time Server - Main entry point

use mcp_utc_time_server::server::McpServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialise logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_utc_time_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Create and run the MCP server
    let mut server = McpServer::new();

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
