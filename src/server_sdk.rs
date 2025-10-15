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
                "MCP UTC Time Server - Provides high-precision time and timezone services.\n\n\
                 Tools: get_time, get_unix_time, get_nanos, get_time_formatted, get_time_with_timezone, list_timezones, convert_time\n\
                 Prompts: /time, /unix_time, /time_in <timezone>, /format_time <format>"
                    .into(),
            ),
        }
    }
}

/// Run the server
pub async fn run() -> Result<()> {
    info!("MCP UTC Time Server (SDK) starting");

    let server = TimeServer::new();
    let service = server.serve(stdio()).await?;

    info!("Server ready, waiting for connections");
    service.waiting().await?;

    Ok(())
}
