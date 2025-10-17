// MCP UTC Time Server using official Rust SDK
//
// This implementation uses the official MCP Rust SDK (rmcp) to provide
// time and timezone services following MCP 2025-06-18 specification.

use anyhow::Result;
use rmcp::{
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, info};

use crate::time::utc::EnhancedTimeResponse;
use crate::time::{TimezoneConverter, UnixTime};

// Parameter types for tools and prompts
#[derive(Debug, Deserialize, JsonSchema)]
struct FormatParams {
    format: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TimezoneParams {
    timezone: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ConvertTimeParams {
    timestamp: i64,
    to_timezone: String,
    #[serde(default)]
    from_timezone: Option<String>,
}

/// Time server implementing MCP protocol
#[derive(Clone)]
pub struct TimeServer {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl TimeServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }
}

impl Default for TimeServer {
    fn default() -> Self {
        Self::new()
    }
}

// Tool implementations using macros
#[tool_router]
impl TimeServer {
    /// Get current UTC time with full Unix/POSIX details
    #[tool(description = "Get current UTC time with full Unix/POSIX details")]
    async fn get_time(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: get_time");
        let response = EnhancedTimeResponse::now();
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Get Unix epoch time with nanosecond precision
    #[tool(description = "Get Unix epoch time with nanosecond precision")]
    async fn get_unix_time(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: get_unix_time");
        let unix_time = UnixTime::now();
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&unix_time)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Get nanoseconds since Unix epoch
    #[tool(description = "Get nanoseconds since Unix epoch")]
    async fn get_nanos(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: get_nanos");
        let unix_time = UnixTime::now();
        let result = json!({
            "nanoseconds": unix_time.nanos_since_epoch,
            "seconds": unix_time.seconds,
            "subsec_nanos": unix_time.nanos,
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Get time formatted with strftime format string
    #[tool(
        description = "Get time formatted with strftime format string (e.g., '%Y-%m-%d %H:%M:%S')"
    )]
    async fn get_time_formatted(
        &self,
        Parameters(params): Parameters<FormatParams>,
    ) -> Result<CallToolResult, McpError> {
        let format = params.format;
        debug!("Tool: get_time_formatted with format: {}", format);
        let response = EnhancedTimeResponse::now();
        let formatted = response
            .format_custom(&format)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let result = json!({
            "formatted": formatted,
            "format": format,
            "unix_seconds": response.unix.seconds,
            "unix_nanos": response.unix.nanos,
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Get time in specified timezone
    #[tool(description = "Get time in specified timezone (IANA name like 'America/New_York')")]
    async fn get_time_with_timezone(
        &self,
        Parameters(params): Parameters<TimezoneParams>,
    ) -> Result<CallToolResult, McpError> {
        let timezone = params.timezone;
        debug!("Tool: get_time_with_timezone for {}", timezone);
        let response = EnhancedTimeResponse::with_timezone(&timezone)
            .map_err(|e| McpError::invalid_params(e, None))?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// List all available IANA timezones
    #[tool(description = "List all available IANA timezones")]
    async fn list_timezones(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: list_timezones");
        let timezones = TimezoneConverter::list_timezones();
        let result = json!({
            "timezones": timezones,
            "count": timezones.len(),
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Convert timestamp between timezones
    #[tool(description = "Convert Unix timestamp between timezones")]
    async fn convert_time(
        &self,
        Parameters(params): Parameters<ConvertTimeParams>,
    ) -> Result<CallToolResult, McpError> {
        let timestamp = params.timestamp;
        let to_timezone = params.to_timezone;
        let from_timezone = params.from_timezone;
        let from_tz = from_timezone.as_deref().unwrap_or("UTC");
        debug!("Tool: convert_time from {} to {}", from_tz, to_timezone);

        use chrono::{Offset, TimeZone, Utc};

        let utc = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| McpError::invalid_params("Invalid timestamp".to_string(), None))?;

        let converted = TimezoneConverter::convert_to_tz(utc, &to_timezone)
            .map_err(|e| McpError::invalid_params(e, None))?;

        let result = json!({
            "original": {
                "timestamp": timestamp,
                "timezone": from_tz,
                "formatted": utc.to_rfc3339(),
            },
            "converted": {
                "timestamp": converted.timestamp(),
                "timezone": to_timezone,
                "formatted": converted.to_rfc3339(),
                "offset": converted.offset().fix().local_minus_utc(),
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?,
        )]))
    }

    /// Get NTP synchronization status (read-only)
    #[tool(description = "Get NTP synchronization status and performance metrics (read-only)")]
    async fn get_ntp_status(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: get_ntp_status");

        use crate::ntp::NtpSyncedClock;

        // Check if NTP is available
        let is_synced = NtpSyncedClock::is_synced().unwrap_or(false);

        if !is_synced {
            let result = json!({
                "available": false,
                "message": "NTP not available or not synchronized",
                "synced": false
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?,
            )]));
        }

        // Get detailed NTP status
        match NtpSyncedClock::get_status() {
            Ok(status) => {
                let result = json!({
                    "available": true,
                    "synced": status.synced,
                    "offset_ms": status.offset_ms,
                    "stratum": status.stratum,
                    "precision": status.precision,
                    "root_delay": status.root_delay,
                    "root_dispersion": status.root_dispersion,
                    "health": if status.synced && status.offset_ms.abs() < 100.0 {
                        "healthy"
                    } else if status.synced {
                        "degraded"
                    } else {
                        "unhealthy"
                    }
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                )]))
            }
            Err(e) => {
                let result = json!({
                    "available": false,
                    "error": e,
                    "synced": false
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                )]))
            }
        }
    }

    /// Get NTP peers information (read-only)
    #[tool(description = "Get information about NTP peers and their status (read-only)")]
    async fn get_ntp_peers(&self) -> Result<CallToolResult, McpError> {
        debug!("Tool: get_ntp_peers");

        // Execute ntpq -p to get peer information
        let output = std::process::Command::new("ntpq")
            .args(["-p", "-n"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let result = json!({
                    "available": true,
                    "peers": stdout.lines().collect::<Vec<_>>(),
                    "raw_output": stdout.to_string()
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                )]))
            }
            _ => {
                let result = json!({
                    "available": false,
                    "error": "NTP daemon not available or ntpq command failed"
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?,
                )]))
            }
        }
    }
}

// Prompt implementations
#[prompt_router]
impl TimeServer {
    /// Get current UTC time
    #[prompt(
        name = "time",
        description = "⏰ Get current UTC time with detailed information"
    )]
    async fn prompt_time(&self) -> Vec<PromptMessage> {
        let time_data = EnhancedTimeResponse::now();
        let text = format!(
            "Here is the current UTC time:\n\n{}",
            serde_json::to_string_pretty(&time_data).unwrap_or_else(|_| "Error".to_string())
        );

        vec![PromptMessage::new_text(PromptMessageRole::User, text)]
    }

    /// Get Unix timestamp
    #[prompt(
        name = "unix_time",
        description = "🕐 Get current Unix timestamp with nanosecond precision"
    )]
    async fn prompt_unix_time(&self) -> Vec<PromptMessage> {
        let unix_time = UnixTime::now();
        let text = format!(
            "Here is the current Unix timestamp:\n\n{}",
            serde_json::to_string_pretty(&unix_time).unwrap_or_else(|_| "Error".to_string())
        );

        vec![PromptMessage::new_text(PromptMessageRole::User, text)]
    }

    /// Get time in specific timezone
    #[prompt(
        name = "time_in",
        description = "🌍 Get current time in a specific timezone (IANA name)"
    )]
    async fn prompt_time_in(
        &self,
        Parameters(params): Parameters<TimezoneParams>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let timezone = params.timezone;
        let time_data = EnhancedTimeResponse::with_timezone(&timezone)
            .map_err(|e| McpError::invalid_params(e, None))?;

        let text = format!(
            "Here is the current time in {}:\n\n{}",
            timezone,
            serde_json::to_string_pretty(&time_data).unwrap_or_else(|_| "Error".to_string())
        );

        Ok(vec![PromptMessage::new_text(PromptMessageRole::User, text)])
    }

    /// Format time
    #[prompt(
        name = "format_time",
        description = "📅 Get current time in a custom strftime format"
    )]
    async fn prompt_format_time(
        &self,
        Parameters(params): Parameters<FormatParams>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let format = params.format;
        let response = EnhancedTimeResponse::now();
        let formatted = response
            .format_custom(&format)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let result = json!({
            "formatted": formatted,
            "format": format,
            "unix_seconds": response.unix.seconds,
        });

        let text = format!(
            "Here is the current time formatted as '{}':\n\n{}",
            format,
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Error".to_string())
        );

        Ok(vec![PromptMessage::new_text(PromptMessageRole::User, text)])
    }
}

// Server handler implementation
#[tool_handler]
#[prompt_handler]
impl ServerHandler for TimeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "mcp-utc-time-server".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some(
                "MCP UTC Time Server - Provides high-precision time, timezone, and NTP status services.\n\n\
                 Time Tools: get_time, get_unix_time, get_nanos, get_time_formatted, get_time_with_timezone, list_timezones, convert_time\n\
                 NTP Tools: get_ntp_status, get_ntp_peers\n\
                 Prompts: /time, /unix_time, /time_in <timezone>, /format_time <format>"
                    .into(),
            ),
        }
    }
}

/// Run the MCP server (STDIO transport)
pub async fn run() -> Result<()> {
    info!(
        event = "server.start",
        version = env!("CARGO_PKG_VERSION"),
        transport = "stdio",
        "MCP UTC Time Server starting"
    );

    let server = TimeServer::new();
    let service = server.serve(stdio()).await?;

    info!(
        event = "server.ready",
        "Server ready, waiting for connections"
    );
    service.waiting().await?;

    Ok(())
}

/// Run HTTP health server for Docker health checks
pub async fn run_health_server() -> Result<()> {
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let port = std::env::var("HEALTH_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;

    info!(
        event = "health.server.start",
        port = port,
        "Health server listening"
    );

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            if let Ok(n) = socket.read(&mut buf).await {
                let request = String::from_utf8_lossy(&buf[..n]);

                // Parse HTTP request
                let response = if request.starts_with("GET /health") {
                    // Health check endpoint
                    use crate::ntp::NtpSyncedClock;

                    let ntp_status = NtpSyncedClock::get_status()
                        .map(|s| {
                            json!({
                                "synced": s.synced,
                                "offset_ms": s.offset_ms,
                                "stratum": s.stratum
                            })
                        })
                        .unwrap_or_else(|_| json!({"available": false}));

                    let health = json!({
                        "status": "healthy",
                        "version": env!("CARGO_PKG_VERSION"),
                        "service": "mcp-utc-time-server",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "ntp": ntp_status
                    });

                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                        serde_json::to_string_pretty(&health).unwrap_or_default()
                    )
                } else if request.starts_with("GET /metrics") {
                    // Prometheus metrics endpoint
                    let unix_time = crate::time::UnixTime::now();
                    let metrics = format!(
                        "# HELP mcp_time_seconds Current Unix timestamp\n\
                         # TYPE mcp_time_seconds gauge\n\
                         mcp_time_seconds {}\n\
                         # HELP mcp_time_nanos Current nanoseconds component\n\
                         # TYPE mcp_time_nanos gauge\n\
                         mcp_time_nanos {}\n",
                        unix_time.seconds, unix_time.nanos
                    );
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                        metrics
                    )
                } else {
                    "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n404 Not Found".to_string()
                };

                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
    }
}
