// MCP UTC Time Server Library

pub mod error;
pub mod mcp;
pub mod server;
pub mod time;

// Re-export commonly used types
pub use error::{McpError, Result};
pub use server::McpServer;
pub use time::utc::EnhancedTimeResponse;
pub use time::UnixTime;
