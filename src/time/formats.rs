// C-style strftime format support

use chrono::{DateTime, Utc};
use std::error::Error;

/// Format time using C strftime format strings
pub struct StrftimeFormatter;

impl StrftimeFormatter {
    /// Format time using POSIX strftime format
    /// Supports all standard format specifiers:
    /// %Y - Year (e.g., 2024)
    /// %m - Month (01-12)
    /// %d - Day (01-31)
    /// %H - Hour (00-23)
    /// %M - Minute (00-59)
    /// %S - Second (00-60)
    /// %z - Timezone offset
    /// %Z - Timezone name
    /// %c - Locale's date and time
    /// %s - Unix timestamp
    pub fn format(dt: &DateTime<Utc>, format: &str) -> Result<String, Box<dyn Error>> {
        // Use chrono's strftime-compatible formatting
        Ok(dt.format(format).to_string())
    }
}

/// Common Unix time formats
pub struct StandardFormats;

impl StandardFormats {
    pub const ISO_8601: &'static str = "%Y-%m-%dT%H:%M:%S%.f%:z";
    pub const RFC_3339: &'static str = "%Y-%m-%d %H:%M:%S%.f %:z";
    pub const RFC_2822: &'static str = "%a, %d %b %Y %H:%M:%S %z";
    pub const CTIME: &'static str = "%c"; // Locale's date and time
    pub const UNIX_DATE: &'static str = "%a %b %e %H:%M:%S %Z %Y";
    pub const SYSLOG: &'static str = "%b %d %H:%M:%S";
    pub const APACHE_LOG: &'static str = "%d/%b/%Y:%H:%M:%S %z";
    pub const UNIX_TIMESTAMP: &'static str = "%s";
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Utc};

    #[test]
    fn test_strftime_formats() {
        let now = Utc::now();

        // Test standard formats
        let iso = StrftimeFormatter::format(&now, StandardFormats::ISO_8601).unwrap();
        assert!(iso.contains("T"));

        let unix_date = StrftimeFormatter::format(&now, StandardFormats::UNIX_DATE).unwrap();
        assert!(unix_date.contains(&now.year().to_string()));

        // Test custom format
        let custom = StrftimeFormatter::format(&now, "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(custom.len(), 19);
    }
}
