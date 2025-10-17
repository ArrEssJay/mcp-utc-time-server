// NTP Integration Module
pub mod config;
pub mod sync;

pub use config::NtpConfig;
pub use sync::{NtpStatus, NtpSyncedClock};
