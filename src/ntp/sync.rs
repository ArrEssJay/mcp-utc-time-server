// NTP-synchronized clock access via NTPsec shared memory interface
use libc::{shmat, shmdt, shmget, IPC_CREAT};
use std::ptr;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const NTP_SHM_SIZE: usize = 96;

/// NTPsec shared memory structure for time exchange
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NtpShmTime {
    mode: i32,                    // 0: both RW, 1: RW by ntpd, RO by us
    count: i32,                   // Updated each write
    clock_time_sec: i64,          // Clock timestamp seconds
    clock_time_usec: i32,         // Clock timestamp microseconds
    receive_time_sec: i64,        // When timestamp was received
    receive_time_usec: i32,       // Receive time microseconds
    leap: i32,                    // Leap second indicator
    precision: i32,               // Clock precision (log2 seconds)
    nsamples: i32,                // Number of samples
    valid: i32,                   // 0: invalid, 1: valid
    clock_time_stamp_nsec: u32,   // Nanosecond resolution
    receive_time_stamp_nsec: u32, // Nanosecond resolution
    dummy: [i32; 8],              // Reserved for future use
}

#[derive(Debug, Clone)]
pub struct NtpStatus {
    pub synced: bool,
    pub offset_ms: f64,
    pub stratum: u8,
    pub precision: i8,
    pub root_delay: f64,
    pub root_dispersion: f64,
    pub shm_valid: bool,
    pub pps_enabled: bool,
}

/// Shared memory interface to NTPsec
pub struct NtpShmInterface {
    #[allow(dead_code)] // Used in Drop implementation
    shm_id: i32,
    shm_ptr: *mut NtpShmTime,
    unit: u8,
}

impl NtpShmInterface {
    /// Create SHM interface for NTPsec unit 0-3
    /// Unit 0 corresponds to SHM(0) in ntp.conf, uses key 0x4e545030
    /// Unit 1 corresponds to SHM(1) in ntp.conf, uses key 0x4e545031, etc.
    pub fn new(unit: u8) -> Result<Self, String> {
        if unit > 3 {
            return Err("SHM unit must be 0-3".to_string());
        }

        // NTPsec uses magic keys: 0x4e545030 + unit number
        let key = 0x4e545030 + unit as i32;

        unsafe {
            // Get or create shared memory segment
            let shm_id = shmget(key, NTP_SHM_SIZE, IPC_CREAT | 0o666);
            if shm_id < 0 {
                return Err(format!(
                    "Failed to create SHM segment for unit {}: {}",
                    unit,
                    std::io::Error::last_os_error()
                ));
            }

            // Attach to shared memory
            let shm_ptr = shmat(shm_id, ptr::null(), 0) as *mut NtpShmTime;
            if shm_ptr as isize == -1 {
                return Err(format!(
                    "Failed to attach SHM segment: {}",
                    std::io::Error::last_os_error()
                ));
            }

            // Initialize the structure if it's new
            let shm = &mut *shm_ptr;
            if shm.mode == 0 && shm.count == 0 {
                *shm = NtpShmTime {
                    mode: 1, // Mode 1: ntpd writes, we read
                    count: 0,
                    clock_time_sec: 0,
                    clock_time_usec: 0,
                    receive_time_sec: 0,
                    receive_time_usec: 0,
                    leap: 0,
                    precision: -20, // Microsecond precision
                    nsamples: 0,
                    valid: 0,
                    clock_time_stamp_nsec: 0,
                    receive_time_stamp_nsec: 0,
                    dummy: [0; 8],
                };
            }

            Ok(NtpShmInterface {
                shm_id,
                shm_ptr,
                unit,
            })
        }
    }

    /// Read current time data from shared memory
    pub fn read_time(&self) -> Option<(i64, u32, bool)> {
        unsafe {
            let shm = &*self.shm_ptr;

            if shm.valid == 0 {
                return None;
            }

            // Memory barrier to ensure reads are consistent
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);

            Some((
                shm.clock_time_sec,
                shm.clock_time_stamp_nsec,
                shm.valid == 1,
            ))
        }
    }

    /// Get the unit number
    pub fn unit(&self) -> u8 {
        self.unit
    }

    /// Check if the shared memory has valid data
    pub fn is_valid(&self) -> bool {
        unsafe {
            let shm = &*self.shm_ptr;
            shm.valid == 1
        }
    }
}

impl Drop for NtpShmInterface {
    fn drop(&mut self) {
        unsafe {
            shmdt(self.shm_ptr as *const libc::c_void);
        }
    }
}

unsafe impl Send for NtpShmInterface {}
unsafe impl Sync for NtpShmInterface {}

pub struct NtpSyncedClock {
    shm: Option<NtpShmInterface>,
}

impl NtpSyncedClock {
    /// Check if running in a container environment
    pub fn is_container_environment() -> bool {
        std::path::Path::new("/.dockerenv").exists()
            || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::env::var("CONTAINER_APP_NAME").is_ok()
            || std::env::var("SKIP_NTP_CHECK").is_ok()
    }

    /// Create a new NTP synced clock with optional SHM interface
    pub fn new() -> Self {
        // Try to connect to SHM(0) by default
        let shm = NtpShmInterface::new(0).ok();
        Self { shm }
    }

    /// Create with specific SHM unit
    pub fn with_shm_unit(unit: u8) -> Result<Self, String> {
        let shm = NtpShmInterface::new(unit)?;
        Ok(Self { shm: Some(shm) })
    }

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
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            Ok((now.as_secs() as i64, now.subsec_nanos()))
        }
    }

    /// Get time from SHM if available, otherwise fallback to system time
    pub fn now_synced(&self) -> Result<(i64, u32), std::io::Error> {
        if let Some(ref shm) = self.shm {
            if let Some((secs, nanos, _)) = shm.read_time() {
                return Ok((secs, nanos));
            }
        }

        // Fallback to system time
        Self::now()
    }

    /// Wait for NTP synchronization
    pub async fn wait_for_sync(timeout_duration: Duration) -> Result<(), String> {
        let start = tokio::time::Instant::now();

        loop {
            if Self::is_synced_async().await? {
                return Ok(());
            }

            if start.elapsed() > timeout_duration {
                return Err("NTP sync timeout".to_string());
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Check NTP synchronization status (async, container-aware)
    pub async fn is_synced_async() -> Result<bool, String> {
        // In containers, skip NTP check
        if Self::is_container_environment() {
            tracing::debug!("Container environment detected, skipping NTP check");
            return Ok(false);
        }

        // Add timeout to prevent indefinite hangs
        let result = timeout(
            Duration::from_secs(2),
            Command::new("ntpq").args(["-p", "-n"]).output(),
        )
        .await;

        let output = match result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!("ntpq not found, assuming not synced");
                return Ok(false);
            }
            Ok(Err(e)) => return Err(format!("Failed to check NTP status: {}", e)),
            Err(_) => {
                tracing::warn!("ntpq command timed out");
                return Ok(false);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for synchronized peer (marked with *)
        Ok(stdout.lines().any(|line| line.starts_with('*')))
    }

    /// Check NTP synchronization status (deprecated blocking version)
    #[deprecated(note = "Use is_synced_async() instead to avoid blocking")]
    pub fn is_synced() -> Result<bool, String> {
        // Fallback for backward compatibility
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::is_synced_async())
        })
    }

    /// Get NTP status information (async, container-aware)
    pub async fn get_status_async(&self) -> Result<NtpStatus, String> {
        // In container environment, return minimal status
        if Self::is_container_environment() {
            tracing::debug!("Container environment: returning degraded NTP status");
            return Ok(NtpStatus {
                synced: true, // Assume host time is good
                offset_ms: 0.0,
                stratum: 3, // Assume container host is synced
                precision: -20,
                root_delay: 0.0,
                root_dispersion: 0.0,
                shm_valid: false,
                pps_enabled: false,
            });
        }

        // Add timeout for ntpq command
        let result = timeout(
            Duration::from_secs(2),
            Command::new("ntpq").args(["-c", "rv"]).output(),
        )
        .await;

        let output = match result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!("ntpq not found, returning degraded status");
                return Ok(NtpStatus {
                    synced: false,
                    offset_ms: 0.0,
                    stratum: 16,
                    precision: 0,
                    root_delay: 0.0,
                    root_dispersion: 0.0,
                    shm_valid: self.shm.as_ref().map(|s| s.is_valid()).unwrap_or(false),
                    pps_enabled: false,
                });
            }
            Ok(Err(e)) => return Err(format!("Failed to get NTP status: {}", e)),
            Err(_) => {
                tracing::warn!("ntpq command timed out, returning degraded status");
                return Ok(NtpStatus {
                    synced: false,
                    offset_ms: 0.0,
                    stratum: 16,
                    precision: 0,
                    root_delay: 0.0,
                    root_dispersion: 0.0,
                    shm_valid: self.shm.as_ref().map(|s| s.is_valid()).unwrap_or(false),
                    pps_enabled: false,
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        let shm_valid = self.shm.as_ref().map(|s| s.is_valid()).unwrap_or(false);
        let pps_enabled = stdout.contains("pps") || stdout.contains("PPS");

        let mut status = NtpStatus {
            synced: Self::is_synced_async().await.unwrap_or(false),
            offset_ms: 0.0,
            stratum: 16,
            precision: 0,
            root_delay: 0.0,
            root_dispersion: 0.0,
            shm_valid,
            pps_enabled,
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

    /// Get NTP status information (deprecated blocking version)
    #[deprecated(note = "Use get_status_async() instead to avoid blocking")]
    pub fn get_status(&self) -> Result<NtpStatus, String> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.get_status_async())
        })
    }

    /// Get NTP offset in microseconds (async)
    pub async fn get_offset_us_async(&self) -> Result<i64, String> {
        let status = self.get_status_async().await?;
        Ok((status.offset_ms * 1000.0) as i64)
    }

    /// Get NTP offset in microseconds (deprecated blocking version)
    #[deprecated(note = "Use get_offset_us_async() instead to avoid blocking")]
    pub fn get_offset_us(&self) -> Result<i64, String> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.get_offset_us_async())
        })
    }
}

impl Default for NtpSyncedClock {
    fn default() -> Self {
        Self::new()
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
