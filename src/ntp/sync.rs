// NTP-synchronized clock access
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NtpStatus {
    pub synced: bool,
    pub offset_ms: f64,
    pub stratum: u8,
    pub precision: i8,
    pub root_delay: f64,
    pub root_dispersion: f64,
}

pub struct NtpSyncedClock;

impl NtpSyncedClock {
    /// Get high-precision system time using clock_gettime
    pub fn now() -> Result<(i64, u32), std::io::Error> {
        #[cfg(unix)]
        {
            use libc::{clock_gettime, timespec, CLOCK_REALTIME};

            let mut ts = timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };

            let result = unsafe { clock_gettime(CLOCK_REALTIME, &mut ts) };

            if result == 0 {
                Ok((ts.tv_sec, ts.tv_nsec as u32))
            } else {
                Err(std::io::Error::last_os_error())
            }
        }

        #[cfg(not(unix))]
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            Ok((now.as_secs() as i64, now.subsec_nanos()))
        }
    }

    /// Wait for NTP synchronization
    pub async fn wait_for_sync(timeout: Duration) -> Result<(), String> {
        let start = tokio::time::Instant::now();

        loop {
            if Self::is_synced()? {
                return Ok(());
            }

            if start.elapsed() > timeout {
                return Err("NTP sync timeout".to_string());
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Check NTP synchronization status
    pub fn is_synced() -> Result<bool, String> {
        // Try to execute ntpq to check sync status
        let output = std::process::Command::new("ntpq")
            .args(["-p", "-n"])
            .output()
            .map_err(|e| format!("Failed to check NTP status: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for synchronized peer (marked with *)
        Ok(stdout.lines().any(|line| line.starts_with('*')))
    }

    /// Get NTP status information
    pub fn get_status() -> Result<NtpStatus, String> {
        let output = std::process::Command::new("ntpq")
            .args(["-c", "rv"])
            .output()
            .map_err(|e| format!("Failed to get NTP status: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut status = NtpStatus {
            synced: Self::is_synced().unwrap_or(false),
            offset_ms: 0.0,
            stratum: 16,
            precision: 0,
            root_delay: 0.0,
            root_dispersion: 0.0,
        };

        // Parse NTP variables
        for part in stdout.split(',') {
            let part = part.trim();

            if part.starts_with("offset=") {
                if let Some(val) = part.strip_prefix("offset=") {
                    status.offset_ms = val.parse().unwrap_or(0.0);
                }
            } else if part.starts_with("stratum=") {
                if let Some(val) = part.strip_prefix("stratum=") {
                    status.stratum = val.parse().unwrap_or(16);
                }
            } else if part.starts_with("precision=") {
                if let Some(val) = part.strip_prefix("precision=") {
                    status.precision = val.parse().unwrap_or(0);
                }
            } else if part.starts_with("rootdelay=") {
                if let Some(val) = part.strip_prefix("rootdelay=") {
                    status.root_delay = val.parse().unwrap_or(0.0);
                }
            } else if part.starts_with("rootdisp=") {
                if let Some(val) = part.strip_prefix("rootdisp=") {
                    status.root_dispersion = val.parse().unwrap_or(0.0);
                }
            }
        }

        Ok(status)
    }

    /// Get NTP offset in microseconds
    pub fn get_offset_us() -> Result<i64, String> {
        let status = Self::get_status()?;
        Ok((status.offset_ms * 1000.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let result = NtpSyncedClock::now();
        assert!(result.is_ok());

        let (secs, nanos) = result.unwrap();
        assert!(secs > 0);
        assert!(nanos < 1_000_000_000);
    }
}
