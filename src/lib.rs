// MCP UTC Time Server Library

pub mod auth;
pub mod ntp;
pub mod server_sdk;
pub mod time;

// Re-export commonly used types
pub use auth::{ApiKey, ApiKeyValidator};
pub use ntp::{NtpConfig, NtpStatus, NtpSyncedClock};
pub use time::utc::EnhancedTimeResponse;
pub use time::UnixTime;
