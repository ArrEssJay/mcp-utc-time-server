// NTP Configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpConfig {
    pub servers: Vec<NtpServer>,
    pub pps: Option<PpsConfig>,
    pub gps: Option<GpsConfig>,
    pub stratum: Option<u8>,
    pub drift_file: String,
    pub stats_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpServer {
    pub host: String,
    pub iburst: bool,
    pub prefer: bool,
    pub minpoll: Option<u8>,
    pub maxpoll: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpsConfig {
    pub enabled: bool,
    pub device: String,
    pub gpio_pin: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsConfig {
    pub enabled: bool,
    pub device: String,
    pub baud_rate: u32,
}

impl Default for NtpConfig {
    fn default() -> Self {
        Self {
            servers: vec![
                NtpServer {
                    host: "time.cloudflare.com".to_string(),
                    iburst: true,
                    prefer: false,
                    minpoll: None,
                    maxpoll: None,
                },
                NtpServer {
                    host: "time.google.com".to_string(),
                    iburst: true,
                    prefer: false,
                    minpoll: None,
                    maxpoll: None,
                },
            ],
            pps: None,
            gps: None,
            stratum: Some(10),
            drift_file: "/var/lib/ntp/ntp.drift".to_string(),
            stats_dir: "/var/log/ntpstats".to_string(),
        }
    }
}

impl NtpConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Parse NTP servers from environment
        if let Ok(servers) = std::env::var("NTP_SERVERS") {
            config.servers = servers
                .split(',')
                .map(|s| NtpServer {
                    host: s.trim().to_string(),
                    iburst: true,
                    prefer: false,
                    minpoll: None,
                    maxpoll: None,
                })
                .collect();
        }

        // Parse PPS configuration
        if let Ok(pps_enabled) = std::env::var("ENABLE_PPS") {
            if pps_enabled == "yes" {
                config.pps = Some(PpsConfig {
                    enabled: true,
                    device: "/dev/pps0".to_string(),
                    gpio_pin: std::env::var("PPS_GPIO").ok().and_then(|s| s.parse().ok()),
                });
            }
        }

        // Parse GPS configuration
        if let Ok(gps_enabled) = std::env::var("ENABLE_GPS") {
            if gps_enabled == "yes" {
                config.gps = Some(GpsConfig {
                    enabled: true,
                    device: std::env::var("GPS_DEVICE")
                        .unwrap_or_else(|_| "/dev/ttyAMA0".to_string()),
                    baud_rate: std::env::var("GPS_BAUD")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(9600),
                });
            }
        }

        // Parse stratum
        if let Ok(stratum) = std::env::var("LOCAL_STRATUM") {
            config.stratum = stratum.parse().ok();
        }

        config
    }
}
