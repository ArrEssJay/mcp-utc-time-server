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

    // Check if we should run HTTP API server alongside MCP server
    let enable_http_api = env::var("ENABLE_HTTP_API")
        .or_else(|_| env::var("ENABLE_HEALTH_SERVER")) // Backward compatibility
        .unwrap_or_else(|_| "true".into())
        .parse::<bool>()
        .unwrap_or(true);

    // Check if we're running in container mode (HTTP API only, no stdio)
    let container_mode = env::var("CONTAINER_APP_NAME").is_ok()
        || env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || env::var("HTTP_API_ONLY").is_ok();

    if container_mode {
        // Container mode: run ONLY the HTTP API server (no stdin available for MCP stdio)
        tracing::info!("Running in container mode - HTTP API server only");
        mcp_utc_time_server::server_sdk::run_http_api_server().await
    } else if enable_http_api {
        // Local mode: run both HTTP API server and MCP stdio server
        tokio::spawn(async {
            if let Err(e) = mcp_utc_time_server::server_sdk::run_http_api_server().await {
                eprintln!("HTTP API server error: {}", e);
            }
        });

        // Run the MCP server with official SDK (STDIO transport)
        mcp_utc_time_server::server_sdk::run().await
    } else {
        // MCP stdio server only
        mcp_utc_time_server::server_sdk::run().await
    }
}
