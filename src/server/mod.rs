// MCP server using STDIO transport

pub mod handlers;
pub mod protocol;

use crate::error::Result;
use crate::mcp::types::{McpRequest, McpResponse};
use handlers::TimeHandler;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

pub struct McpServer {
    handler: TimeHandler,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            handler: TimeHandler::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("MCP UTC Time Server started");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF received, shutting down");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received request: {}", trimmed);

                    match serde_json::from_str::<McpRequest>(trimmed) {
                        Ok(request) => {
                            let response = self.handler.handle_request(request).await;

                            match serde_json::to_string(&response) {
                                Ok(response_str) => {
                                    debug!("Sending response: {}", response_str);
                                    if let Err(e) = stdout.write_all(response_str.as_bytes()).await
                                    {
                                        error!("Failed to write response: {}", e);
                                        break;
                                    }
                                    if let Err(e) = stdout.write_all(b"\n").await {
                                        error!("Failed to write newline: {}", e);
                                        break;
                                    }
                                    if let Err(e) = stdout.flush().await {
                                        error!("Failed to flush stdout: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse request: {}", e);
                            let error_response =
                                McpResponse::error(-32700, format!("Parse error: {}", e), None);
                            if let Ok(response_str) = serde_json::to_string(&error_response) {
                                let _ = stdout.write_all(response_str.as_bytes()).await;
                                let _ = stdout.write_all(b"\n").await;
                                let _ = stdout.flush().await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        info!("MCP UTC Time Server stopped");
        Ok(())
    }
}
