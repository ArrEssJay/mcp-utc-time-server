// Enhanced UTC time response with Unix/POSIX features

use super::{StandardFormats, StrftimeFormatter, TimezoneConverter, UnixTime};
use chrono::{DateTime, Datelike, Offset, SecondsFormat, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTimeResponse {
    // Unix epoch times
    pub unix: UnixTime,

    // Standard formats
    pub iso8601: String,
    pub rfc3339: String,
    pub rfc2822: String,
    pub ctime: String,

    // Nanosecond precision
    pub nanos_since_epoch: i128,
    pub seconds: i64,
    pub microseconds: i64,
    pub milliseconds: i64,

    // Components
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,

    // Timezone info
    pub timezone: String,
    pub offset: i32,

    // Week info
    pub weekday: String,
    pub week_of_year: u32,
    pub day_of_year: u32,

    // Custom formats
    pub custom_formats: HashMap<String, String>,
}

impl EnhancedTimeResponse {
    pub fn now() -> Self {
        let now_utc = Utc::now();
        let unix_time = UnixTime::now();

        let mut custom_formats = HashMap::new();

        // Add common Unix formats
        custom_formats.insert(
            "unix_date".to_string(),
            StrftimeFormatter::format(&now_utc, StandardFormats::UNIX_DATE).unwrap_or_default(),
        );
        custom_formats.insert(
            "syslog".to_string(),
            StrftimeFormatter::format(&now_utc, StandardFormats::SYSLOG).unwrap_or_default(),
        );
        custom_formats.insert(
            "apache_log".to_string(),
            StrftimeFormatter::format(&now_utc, StandardFormats::APACHE_LOG).unwrap_or_default(),
        );
        custom_formats.insert("unix_timestamp".to_string(), unix_time.seconds.to_string());

        Self {
            unix: unix_time.clone(),
            iso8601: now_utc.to_rfc3339_opts(SecondsFormat::Nanos, true),
            rfc3339: now_utc.to_rfc3339(),
            rfc2822: now_utc.to_rfc2822(),
            ctime: now_utc.format("%c").to_string(),

            nanos_since_epoch: unix_time.nanos_since_epoch,
            seconds: unix_time.seconds,
            microseconds: unix_time.to_microseconds(),
            milliseconds: unix_time.to_milliseconds(),

            year: now_utc.year(),
            month: now_utc.month(),
            day: now_utc.day(),
            hour: now_utc.hour(),
            minute: now_utc.minute(),
            second: now_utc.second(),
            nanosecond: now_utc.nanosecond(),

            timezone: "UTC".to_string(),
            offset: 0,

            weekday: now_utc.format("%A").to_string(),
            week_of_year: now_utc.format("%U").to_string().parse().unwrap_or(0),
            day_of_year: now_utc.ordinal(),

            custom_formats,
        }
    }

    pub fn with_timezone(tz: &str) -> Result<Self, String> {
        let now_utc = Utc::now();
        let converted = TimezoneConverter::convert_to_tz(now_utc, tz)?;

        // Create response with converted timezone
        let mut response = Self::now();
        response.timezone = tz.to_string();
        response.offset = converted.offset().fix().local_minus_utc();

        // Update formatted strings with timezone
        response.iso8601 = converted.to_rfc3339_opts(SecondsFormat::Nanos, true);
        response.rfc3339 = converted.to_rfc3339();
        response.rfc2822 = converted.to_rfc2822();

        Ok(response)
    }

    pub fn format_custom(&self, format: &str) -> Result<String, Box<dyn std::error::Error>> {
        let dt = DateTime::<Utc>::from_timestamp(self.unix.seconds, self.unix.nanos)
            .ok_or("Invalid timestamp")?;
        Ok(StrftimeFormatter::format(&dt, format)?)
    }
}

// Legacy function for backwards compatibility
pub fn get_current_utc_time() -> String {
    let utc_time: DateTime<Utc> = Utc::now();
    utc_time.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_time_response() {
        let response = EnhancedTimeResponse::now();

        assert_eq!(response.timezone, "UTC");
        assert!(response.unix.seconds > 0);
        assert!(response.iso8601.contains("T"));
        assert!(response.custom_formats.contains_key("unix_date"));
    }

    #[test]
    fn test_custom_format() {
        let response = EnhancedTimeResponse::now();
        let formatted = response.format_custom("%Y-%m-%d").unwrap();
        assert_eq!(formatted.len(), 10);
    }
}
