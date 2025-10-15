// MCP request handlers for time operations

use crate::error::{McpError, Result};
use crate::mcp::types::{McpRequest, McpResponse, ServerCapabilities, ToolDefinition};
use crate::time::utc::EnhancedTimeResponse;
use crate::time::{TimezoneConverter, UnixTime};
use chrono::{Offset, TimeZone, Utc};
use serde_json::{json, Value};
use tracing::{debug, error};

pub struct TimeHandler;

impl TimeHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.list_tools(request.params).await,
            "time/get" => self.get_time(request.params).await,
            "time/get_with_format" => self.get_time_formatted(request.params).await,
            "time/get_with_timezone" => self.get_time_with_tz(request.params).await,
            "time/get_unix" => self.get_unix_time(request.params).await,
            "time/get_nanos" => self.get_nanos(request.params).await,
            "time/list_timezones" => self.list_timezones(request.params).await,
            "time/convert" => self.convert_time(request.params).await,
            _ => Err(McpError::MethodNotFound(request.method.clone())),
        };

        match result {
            Ok(value) => McpResponse::success(value, request.id),
            Err(e) => {
                error!("Request error: {}", e);
                McpResponse::error(e.code(), e.to_string(), request.id)
            }
        }
    }

    async fn handle_initialize(&self, _params: Value) -> Result<Value> {
        debug!("Handling initialize request");

        let capabilities = ServerCapabilities {
            tools: self.get_tool_definitions(),
        };

        Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "mcp-utc-time-server",
                "version": "0.1.0"
            },
            "capabilities": capabilities
        }))
    }

    async fn list_tools(&self, _params: Value) -> Result<Value> {
        debug!("Listing tools");
        Ok(json!({
            "tools": self.get_tool_definitions()
        }))
    }

    fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "time/get".to_string(),
                description: "Get current UTC time with full Unix/POSIX details".to_string(),
                parameters: None,
            },
            ToolDefinition {
                name: "time/get_unix".to_string(),
                description: "Get Unix epoch time with nanosecond precision".to_string(),
                parameters: None,
            },
            ToolDefinition {
                name: "time/get_nanos".to_string(),
                description: "Get nanoseconds since Unix epoch".to_string(),
                parameters: None,
            },
            ToolDefinition {
                name: "time/get_with_format".to_string(),
                description: "Get time formatted with strftime format string".to_string(),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "description": "strftime format string"
                        }
                    },
                    "required": ["format"]
                })),
            },
            ToolDefinition {
                name: "time/get_with_timezone".to_string(),
                description: "Get time in specified timezone".to_string(),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "timezone": {
                            "type": "string",
                            "description": "IANA timezone name (e.g., 'America/New_York')"
                        }
                    },
                    "required": ["timezone"]
                })),
            },
            ToolDefinition {
                name: "time/list_timezones".to_string(),
                description: "List all available IANA timezones".to_string(),
                parameters: None,
            },
            ToolDefinition {
                name: "time/convert".to_string(),
                description: "Convert timestamp between timezones".to_string(),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "timestamp": {
                            "type": "number",
                            "description": "Unix timestamp in seconds"
                        },
                        "from_timezone": {
                            "type": "string",
                            "description": "Source timezone (optional, defaults to UTC)"
                        },
                        "to_timezone": {
                            "type": "string",
                            "description": "Target timezone"
                        }
                    },
                    "required": ["timestamp", "to_timezone"]
                })),
            },
        ]
    }

    async fn get_time(&self, _params: Value) -> Result<Value> {
        debug!("Getting current time");
        let response = EnhancedTimeResponse::now();
        Ok(serde_json::to_value(response)?)
    }

    async fn get_time_formatted(&self, params: Value) -> Result<Value> {
        let format = params["format"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("format required".to_string()))?;

        debug!("Getting time with format: {}", format);
        let response = EnhancedTimeResponse::now();
        let formatted = response
            .format_custom(format)
            .map_err(|e| McpError::TimeError(e.to_string()))?;

        Ok(json!({
            "formatted": formatted,
            "format": format,
            "unix_seconds": response.unix.seconds,
            "unix_nanos": response.unix.nanos,
        }))
    }

    async fn get_time_with_tz(&self, params: Value) -> Result<Value> {
        let timezone = params["timezone"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("timezone required".to_string()))?;

        debug!("Getting time for timezone: {}", timezone);
        let response = EnhancedTimeResponse::with_timezone(timezone)
            .map_err(|e| McpError::InvalidParams(e))?;

        Ok(serde_json::to_value(response)?)
    }

    async fn get_unix_time(&self, _params: Value) -> Result<Value> {
        debug!("Getting Unix time");
        let unix_time = UnixTime::now();
        Ok(serde_json::to_value(unix_time)?)
    }

    async fn get_nanos(&self, _params: Value) -> Result<Value> {
        debug!("Getting nanoseconds");
        let unix_time = UnixTime::now();
        Ok(json!({
            "nanoseconds": unix_time.nanos_since_epoch,
            "seconds": unix_time.seconds,
            "subsec_nanos": unix_time.nanos,
        }))
    }

    async fn list_timezones(&self, _params: Value) -> Result<Value> {
        debug!("Listing timezones");
        let timezones = TimezoneConverter::list_timezones();
        Ok(json!({
            "timezones": timezones,
            "count": timezones.len(),
        }))
    }

    async fn convert_time(&self, params: Value) -> Result<Value> {
        let timestamp = params["timestamp"]
            .as_i64()
            .ok_or_else(|| McpError::InvalidParams("timestamp required".to_string()))?;

        let from_tz = params["from_timezone"].as_str().unwrap_or("UTC");
        let to_tz = params["to_timezone"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("to_timezone required".to_string()))?;

        debug!("Converting time from {} to {}", from_tz, to_tz);

        // Perform conversion
        let utc = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| McpError::InvalidParams("Invalid timestamp".to_string()))?;

        let converted =
            TimezoneConverter::convert_to_tz(utc, to_tz).map_err(|e| McpError::InvalidParams(e))?;

        Ok(json!({
            "original": {
                "timestamp": timestamp,
                "timezone": from_tz,
                "formatted": utc.to_rfc3339(),
            },
            "converted": {
                "timestamp": converted.timestamp(),
                "timezone": to_tz,
                "formatted": converted.to_rfc3339(),
                "offset": converted.offset().fix().local_minus_utc(),
            }
        }))
    }
}
