// MCP request handlers for time operations

use crate::error::{McpError, Result};
use crate::mcp::types::{
    McpRequest, McpResponse, PromptArgument, PromptDefinition, PromptsCapability,
    ServerCapabilities, ToolDefinition, ToolsCapability,
};
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
        // Handle notifications (no response needed, but we shouldn't error)
        if request.method.starts_with("notifications/") {
            debug!("Received notification: {}", request.method);
            // Notifications don't get responses, but we return success for logging
            return McpResponse::success(json!({}), request.id);
        }

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.list_tools(request.params).await,
            "tools/call" => self.call_tool(request.params).await,
            "prompts/list" => self.list_prompts(request.params).await,
            "prompts/get" => self.get_prompt(request.params).await,
            // Legacy direct methods (for backward compatibility)
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
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
            prompts: Some(PromptsCapability {
                list_changed: Some(false),
            }),
            resources: None, // Not implementing resources for this time server
        };

        Ok(json!({
            "protocolVersion": "2025-06-18",
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
                name: "get_time".to_string(),
                title: Some("Get Current Time".to_string()),
                description: "Get current UTC time with full Unix/POSIX details".to_string(),
                input_schema: None,
            },
            ToolDefinition {
                name: "get_unix_time".to_string(),
                title: Some("Get Unix Time".to_string()),
                description: "Get Unix epoch time with nanosecond precision".to_string(),
                input_schema: None,
            },
            ToolDefinition {
                name: "get_nanos".to_string(),
                title: Some("Get Nanoseconds".to_string()),
                description: "Get nanoseconds since Unix epoch".to_string(),
                input_schema: None,
            },
            ToolDefinition {
                name: "get_time_formatted".to_string(),
                title: Some("Get Formatted Time".to_string()),
                description: "Get time formatted with strftime format string".to_string(),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "description": "strftime format string (e.g., '%Y-%m-%d %H:%M:%S')"
                        }
                    },
                    "required": ["format"]
                })),
            },
            ToolDefinition {
                name: "get_time_with_timezone".to_string(),
                title: Some("Get Time in Timezone".to_string()),
                description: "Get time in specified timezone".to_string(),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "timezone": {
                            "type": "string",
                            "description": "IANA timezone name (e.g., 'America/New_York', 'Europe/London')"
                        }
                    },
                    "required": ["timezone"]
                })),
            },
            ToolDefinition {
                name: "list_timezones".to_string(),
                title: Some("List Timezones".to_string()),
                description: "List all available IANA timezones".to_string(),
                input_schema: None,
            },
            ToolDefinition {
                name: "convert_time".to_string(),
                title: Some("Convert Time".to_string()),
                description: "Convert timestamp between timezones".to_string(),
                input_schema: Some(json!({
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

    async fn call_tool(&self, params: Value) -> Result<Value> {
        let name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("tool name required".to_string()))?;

        let arguments = params.get("arguments").unwrap_or(&Value::Null).clone();

        debug!("Calling tool: {}", name);

        // Call the appropriate tool based on name
        let result = match name {
            "get_time" => self.get_time(Value::Null).await?,
            "get_unix_time" => self.get_unix_time(Value::Null).await?,
            "get_nanos" => self.get_nanos(Value::Null).await?,
            "get_time_formatted" => self.get_time_formatted(arguments).await?,
            "get_time_with_timezone" => self.get_time_with_tz(arguments).await?,
            "list_timezones" => self.list_timezones(Value::Null).await?,
            "convert_time" => self.convert_time(arguments).await?,
            _ => {
                return Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Unknown tool: {}", name)
                    }],
                    "isError": true
                }));
            }
        };

        // Convert result to MCP tool call format
        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result)?
            }],
            "isError": false
        }))
    }

    async fn list_prompts(&self, _params: Value) -> Result<Value> {
        debug!("Listing prompts");

        let prompts = vec![
            PromptDefinition {
                name: "time".to_string(),
                title: Some("⏰ Current Time".to_string()),
                description: Some("Get the current UTC time with detailed information".to_string()),
                arguments: None,
            },
            PromptDefinition {
                name: "time_in".to_string(),
                title: Some("🌍 Time in Timezone".to_string()),
                description: Some("Get the current time in a specific timezone".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "timezone".to_string(),
                    description: Some("IANA timezone name (e.g., 'America/New_York')".to_string()),
                    required: Some(true),
                }]),
            },
            PromptDefinition {
                name: "format_time".to_string(),
                title: Some("📅 Format Time".to_string()),
                description: Some("Get the current time in a custom format".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "format".to_string(),
                    description: Some(
                        "strftime format string (e.g., '%Y-%m-%d %H:%M:%S')".to_string(),
                    ),
                    required: Some(true),
                }]),
            },
            PromptDefinition {
                name: "unix_time".to_string(),
                title: Some("🕐 Unix Timestamp".to_string()),
                description: Some(
                    "Get the current Unix timestamp with nanosecond precision".to_string(),
                ),
                arguments: None,
            },
        ];

        Ok(json!({
            "prompts": prompts
        }))
    }

    async fn get_prompt(&self, params: Value) -> Result<Value> {
        let name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("prompt name required".to_string()))?;

        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

        debug!("Getting prompt: {}", name);

        match name {
            "time" => {
                let time_data = self.get_time(Value::Null).await?;
                Ok(json!({
                    "description": "Current UTC time with full details",
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Here is the current UTC time:\n\n{}",
                                serde_json::to_string_pretty(&time_data)?)
                        }
                    }]
                }))
            }
            "time_in" => {
                let timezone = arguments["timezone"]
                    .as_str()
                    .ok_or_else(|| McpError::InvalidParams("timezone required".to_string()))?;

                let time_data = self
                    .get_time_with_tz(json!({ "timezone": timezone }))
                    .await?;
                Ok(json!({
                    "description": format!("Current time in {}", timezone),
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Here is the current time in {}:\n\n{}",
                                timezone, serde_json::to_string_pretty(&time_data)?)
                        }
                    }]
                }))
            }
            "format_time" => {
                let format = arguments["format"]
                    .as_str()
                    .ok_or_else(|| McpError::InvalidParams("format required".to_string()))?;

                let time_data = self.get_time_formatted(json!({ "format": format })).await?;
                Ok(json!({
                    "description": format!("Time formatted as '{}'", format),
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Here is the current time formatted as '{}':\n\n{}",
                                format, serde_json::to_string_pretty(&time_data)?)
                        }
                    }]
                }))
            }
            "unix_time" => {
                let time_data = self.get_unix_time(Value::Null).await?;
                Ok(json!({
                    "description": "Current Unix timestamp",
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Here is the current Unix timestamp:\n\n{}",
                                serde_json::to_string_pretty(&time_data)?)
                        }
                    }]
                }))
            }
            _ => Err(McpError::InvalidParams(format!("Unknown prompt: {}", name))),
        }
    }
}
