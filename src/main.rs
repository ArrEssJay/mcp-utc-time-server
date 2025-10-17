// MCP UTC Time Server - Main entry point

use anyhow::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging to stderr without ANSI colors
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

    // Check if we should run health server alongside MCP server
    let enable_health = env::var("ENABLE_HEALTH_SERVER")
        .unwrap_or_else(|_| "true".into())
        .parse::<bool>()
        .unwrap_or(true);

    if enable_health {
        // Spawn HTTP health server in background
        tokio::spawn(async {
            if let Err(e) = mcp_utc_time_server::server_sdk::run_health_server().await {
                eprintln!("Health server error: {}", e);
            }
        });
    }

    // Run the MCP server with official SDK (STDIO transport)
    mcp_utc_time_server::server_sdk::run().await
}
