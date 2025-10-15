// MCP UTC Time Server - Main entry point

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr without ANSI colors
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_utc_time_server=info,rmcp=warn".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false) // Disable ANSI color codes
                .with_target(false) // Disable target module names
                .compact(), // Use compact format
        )
        .init();

    // Run the server with official SDK
    mcp_utc_time_server::server_sdk::run().await
}
