// Integration tests for MCP UTC Time Server

use chrono::{Datelike, Offset, Utc};
use mcp_utc_time_server::time::utc::EnhancedTimeResponse;
use mcp_utc_time_server::time::{StandardFormats, StrftimeFormatter, TimezoneConverter, UnixTime};

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
fn test_enhanced_time_response() {
    let response = EnhancedTimeResponse::now();

    assert_eq!(response.timezone, "UTC");
    assert!(response.unix.seconds > 0);
    assert!(response.iso8601.contains("T"));
    assert!(response.custom_formats.contains_key("unix_date"));
    assert!(response.custom_formats.contains_key("syslog"));
}

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
    assert!(timezones.contains(&"Asia/Tokyo".to_string()));
}

#[test]
fn test_custom_format() {
    let response = EnhancedTimeResponse::now();

    // Test various custom formats
    let date_only = response.format_custom("%Y-%m-%d").unwrap();
    assert_eq!(date_only.len(), 10);

    let time_only = response.format_custom("%H:%M:%S").unwrap();
    assert_eq!(time_only.len(), 8);

    let unix_timestamp = response.format_custom("%s").unwrap();
    assert!(unix_timestamp.parse::<i64>().is_ok());
}

#[test]
fn test_time_components() {
    let response = EnhancedTimeResponse::now();

    // Verify components are valid
    assert!(response.year >= 2024);
    assert!(response.month >= 1 && response.month <= 12);
    assert!(response.day >= 1 && response.day <= 31);
    assert!(response.hour < 24);
    assert!(response.minute < 60);
    assert!(response.second < 61); // Leap seconds
    assert!(response.nanosecond < 1_000_000_000);
}

#[test]
fn test_time_conversions() {
    let unix_time = UnixTime::now();

    let micros = unix_time.to_microseconds();
    let millis = unix_time.to_milliseconds();

    assert!(micros > unix_time.seconds * 1_000_000);
    assert!(millis > unix_time.seconds * 1000);

    // Verify consistency
    assert_eq!(millis, micros / 1000);
}

#[tokio::test]
async fn test_enhanced_time_with_timezone() {
    let response = EnhancedTimeResponse::with_timezone("America/New_York").unwrap();
    assert_eq!(response.timezone, "America/New_York");
    assert_ne!(response.offset, 0); // Should have offset from UTC

    let response_utc = EnhancedTimeResponse::now();
    // Same timestamp, different representation
    assert_eq!(response.unix.seconds, response_utc.unix.seconds);
}
