// Unix timestamp with nanosecond precision

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unix timestamp with nanosecond precision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixTime {
    /// Seconds since Unix epoch (1970-01-01 00:00:00 UTC)
    pub seconds: i64,
    /// Nanoseconds within the current second (0-999999999)
    pub nanos: u32,
    /// Combined nanoseconds since epoch
    pub nanos_since_epoch: i128,
}

impl UnixTime {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch");

        Self {
            seconds: duration.as_secs() as i64,
            nanos: duration.subsec_nanos(),
            nanos_since_epoch: duration.as_nanos() as i128,
        }
    }

    pub fn to_timespec(&self) -> libc::timespec {
        libc::timespec {
            tv_sec: self.seconds,
            tv_nsec: self.nanos as libc::c_long,
        }
    }

    pub fn to_microseconds(&self) -> i64 {
        self.seconds * 1_000_000 + (self.nanos as i64 / 1000)
    }

    pub fn to_milliseconds(&self) -> i64 {
        self.seconds * 1000 + (self.nanos as i64 / 1_000_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_time_precision() {
        let unix_time = UnixTime::now();

        // Verify nanosecond precision
        assert!(unix_time.nanos < 1_000_000_000);
        assert!(unix_time.nanos_since_epoch > 0);

        // Verify conversion to timespec
        let timespec = unix_time.to_timespec();
        assert_eq!(timespec.tv_sec, unix_time.seconds);
        assert_eq!(timespec.tv_nsec as u32, unix_time.nanos);
    }

    #[test]
    fn test_time_conversions() {
        let unix_time = UnixTime::now();

        let micros = unix_time.to_microseconds();
        let millis = unix_time.to_milliseconds();

        assert!(micros > unix_time.seconds * 1_000_000);
        assert!(millis > unix_time.seconds * 1000);
    }
}
