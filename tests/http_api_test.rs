// HTTP API Integration Tests
// These tests spawn a real HTTP server and test the endpoints
// Tests run sequentially to avoid port conflicts

use serial_test::serial;
use std::time::Duration;
use tokio::time::sleep;

const TEST_PORT: u16 = 13000;

/// Start the HTTP server in the background for testing
async fn start_test_server() -> tokio::task::JoinHandle<()> {
    std::env::set_var("HEALTH_PORT", TEST_PORT.to_string());
    std::env::set_var("CONTAINER_APP_NAME", "test"); // Enable container mode

    tokio::spawn(async {
        if let Err(e) = mcp_utc_time_server::server_sdk::run_health_server().await {
            eprintln!("Server error: {}", e);
        }
    })
}

/// Helper to make HTTP GET requests
async fn get_request(path: &str) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{}{}", TEST_PORT, path);

    match reqwest::get(&url).await {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.map_err(|e| e.to_string())?;

            if status.is_success() {
                Ok(body)
            } else {
                Err(format!("HTTP {} - {}", status, body))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tokio::test]
#[serial]
async fn test_health_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await; // Let server start

    let response = get_request("/health").await;
    assert!(response.is_ok(), "Health check failed: {:?}", response);

    let body = response.unwrap();
    assert!(
        body.contains("healthy"),
        "Response should contain 'healthy'"
    );
    assert!(body.contains("version"), "Response should contain version");
}

#[tokio::test]
#[serial]
async fn test_api_time_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/time").await;
    assert!(response.is_ok(), "GET /api/time failed: {:?}", response);

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    // Check for expected fields
    assert!(json.get("unix").is_some(), "Should have 'unix' field");
    assert!(json.get("rfc3339").is_some(), "Should have 'rfc3339' field");
    assert!(json.get("iso8601").is_some(), "Should have 'iso8601' field");
}

#[tokio::test]
#[serial]
async fn test_api_unix_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/unix").await;
    assert!(response.is_ok(), "GET /api/unix failed: {:?}", response);

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    assert!(json.get("seconds").is_some(), "Should have 'seconds' field");
    assert!(json.get("nanos").is_some(), "Should have 'nanos' field");

    // Verify seconds is a reasonable value (after 2020)
    let seconds = json["seconds"].as_i64().expect("seconds should be i64");
    assert!(
        seconds > 1_600_000_000,
        "Unix timestamp should be after 2020"
    );
}

#[tokio::test]
#[serial]
async fn test_api_nanos_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/nanos").await;
    assert!(response.is_ok(), "GET /api/nanos failed: {:?}", response);

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    assert!(
        json.get("nanoseconds").is_some(),
        "Should have 'nanoseconds' field"
    );
    assert!(json.get("seconds").is_some(), "Should have 'seconds' field");
    assert!(
        json.get("subsec_nanos").is_some(),
        "Should have 'subsec_nanos' field"
    );
}

#[tokio::test]
#[serial]
async fn test_api_timezones_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/timezones").await;
    assert!(
        response.is_ok(),
        "GET /api/timezones failed: {:?}",
        response
    );

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    assert!(
        json.get("timezones").is_some(),
        "Should have 'timezones' field"
    );
    assert!(json.get("count").is_some(), "Should have 'count' field");

    let timezones = json["timezones"]
        .as_array()
        .expect("timezones should be array");
    assert!(!timezones.is_empty(), "Should have at least one timezone");

    // Check for common timezones
    let tz_strings: Vec<String> = timezones
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        tz_strings.contains(&"UTC".to_string()),
        "Should include UTC"
    );
    assert!(
        tz_strings.contains(&"America/New_York".to_string()),
        "Should include America/New_York"
    );
}

#[tokio::test]
#[serial]
async fn test_api_timezone_specific() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/time/timezone/America/New_York").await;
    assert!(
        response.is_ok(),
        "GET /api/time/timezone/America/New_York failed: {:?}",
        response
    );

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    assert!(
        json.get("timezone").is_some(),
        "Should have 'timezone' field"
    );
    assert_eq!(
        json["timezone"].as_str(),
        Some("America/New_York"),
        "Timezone should match"
    );
}

#[tokio::test]
#[serial]
async fn test_api_timezone_invalid() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/time/timezone/Invalid/Zone").await;
    assert!(response.is_err(), "Invalid timezone should return error");
}

#[tokio::test]
#[serial]
async fn test_api_ntp_status_container_mode() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/api/ntp/status").await;
    assert!(
        response.is_ok(),
        "GET /api/ntp/status failed: {:?}",
        response
    );

    let body = response.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    // In container mode, NTP should not be available
    assert_eq!(
        json["available"], false,
        "NTP should not be available in container mode"
    );
    assert_eq!(
        json["container_mode"], true,
        "Should indicate container mode"
    );
}

#[tokio::test]
#[serial]
async fn test_metrics_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let response = get_request("/metrics").await;
    assert!(response.is_ok(), "GET /metrics failed: {:?}", response);

    let body = response.unwrap();
    assert!(
        body.contains("mcp_time_seconds"),
        "Should contain mcp_time_seconds metric"
    );
    assert!(
        body.contains("mcp_time_nanos"),
        "Should contain mcp_time_nanos metric"
    );
    assert!(
        body.contains("# TYPE"),
        "Should contain Prometheus TYPE directives"
    );
}

#[tokio::test]
#[serial]
async fn test_404_endpoint() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let url = format!("http://127.0.0.1:{}/api/nonexistent", TEST_PORT);
    let response = reqwest::get(&url).await.expect("Request failed");

    assert_eq!(
        response.status(),
        404,
        "Should return 404 for unknown endpoint"
    );

    let body = response.text().await.expect("Failed to read body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Invalid JSON");

    assert!(json.get("error").is_some(), "Should have 'error' field");
    assert!(
        json.get("available_endpoints").is_some(),
        "Should list available endpoints"
    );
}

#[tokio::test]
#[serial]
async fn test_cors_headers() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    let url = format!("http://127.0.0.1:{}/api/time", TEST_PORT);
    let response = reqwest::get(&url).await.expect("Request failed");

    let headers = response.headers();
    assert!(
        headers.contains_key("access-control-allow-origin"),
        "Should have CORS header"
    );
}

#[tokio::test]
#[serial]
async fn test_concurrent_requests() {
    let _server = start_test_server().await;
    sleep(Duration::from_millis(500)).await;

    // Make 10 concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        handles.push(tokio::spawn(async { get_request("/api/time").await }));
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Concurrent request failed: {:?}", result);
    }
}
