// STDIO transport layer for MCP protocol
// The actual transport is implemented in server/mod.rs
// This file exports transport-related types for the MCP module

/// STDIO transport for MCP protocol
/// The actual implementation is in src/server/mod.rs which uses tokio's async I/O
#[derive(Debug, Clone, Default)]
pub struct StdioTransport;

impl StdioTransport {
    pub fn new() -> Self {
        Self
    }
}
