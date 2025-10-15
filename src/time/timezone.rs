// Timezone support and conversion

use chrono::{DateTime, FixedOffset, Offset, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneInfo {
    pub name: String,
    pub offset_seconds: i32,
    pub abbreviation: String,
    pub is_dst: bool,
}

pub struct TimezoneConverter;

impl TimezoneConverter {
    /// Convert UTC time to specified timezone
    pub fn convert_to_tz(utc: DateTime<Utc>, timezone: &str) -> Result<DateTime<Tz>, String> {
        let tz: Tz = timezone
            .parse()
            .map_err(|_| format!("Invalid timezone: {}", timezone))?;
        Ok(utc.with_timezone(&tz))
    }

    /// Get all available timezones
    pub fn list_timezones() -> Vec<String> {
        TZ_VARIANTS.iter().map(|tz| tz.to_string()).collect()
    }

    /// Get timezone info for a given timezone
    pub fn get_timezone_info(timezone: &str) -> Result<TimezoneInfo, String> {
        let tz: Tz = timezone
            .parse()
            .map_err(|_| format!("Invalid timezone: {}", timezone))?;

        let now = Utc::now().with_timezone(&tz);
        let offset = now.offset();

        Ok(TimezoneInfo {
            name: timezone.to_string(),
            offset_seconds: offset.fix().local_minus_utc(),
            abbreviation: format!("{}", offset),
            is_dst: false, // Would need additional logic for DST detection
        })
    }

    /// Convert using POSIX TZ string (e.g., "PST8PDT,M3.2.0,M11.1.0")
    pub fn from_posix_tz(
        utc: DateTime<Utc>,
        tz_string: &str,
    ) -> Result<DateTime<FixedOffset>, String> {
        // Parse POSIX TZ string and apply offset
        // This is a simplified implementation
        let offset_hours = if tz_string.contains("EST") {
            -5
        } else if tz_string.contains("PST") {
            -8
        } else if tz_string.contains("GMT") || tz_string.contains("UTC") {
            0
        } else {
            return Err("Unsupported TZ string".to_string());
        };

        let offset = FixedOffset::west_opt(offset_hours * 3600).ok_or("Invalid offset")?;
        Ok(utc.with_timezone(&offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_timezone_conversion() {
        let utc = Utc::now();

        // Test conversion to different timezones
        let eastern = TimezoneConverter::convert_to_tz(utc, "America/New_York").unwrap();
        let tokyo = TimezoneConverter::convert_to_tz(utc, "Asia/Tokyo").unwrap();

        // Verify timestamps are equal (same moment in time)
        assert_eq!(utc.timestamp(), eastern.timestamp());
        assert_eq!(utc.timestamp(), tokyo.timestamp());

        // Verify offsets are different
        // Verify offsets are different
        assert_ne!(
            eastern.offset().fix().local_minus_utc(),
            tokyo.offset().fix().local_minus_utc()
        );
    }

    #[test]
    fn test_list_timezones() {
        let timezones = TimezoneConverter::list_timezones();
        assert!(timezones.len() > 100);
        assert!(timezones.contains(&"America/New_York".to_string()));
        assert!(timezones.contains(&"Europe/London".to_string()));
    }
}
