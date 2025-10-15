// Custom error types for MCP time server

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Time error: {0}")]
    TimeError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            McpError::ParseError(_) => -32700,
            McpError::InvalidRequest(_) => -32600,
            McpError::MethodNotFound(_) => -32601,
            McpError::InvalidParams(_) => -32602,
            McpError::InternalError(_) => -32603,
            _ => -32000,
        }
    }
}

pub type Result<T> = std::result::Result<T, McpError>;
