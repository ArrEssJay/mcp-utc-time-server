# HTTP API Testing Guide

## Running Tests Locally

The HTTP API tests can run locally without Docker. They test all REST endpoints by spawning a real HTTP server.

### Quick Start

```bash
# Run all HTTP API tests
./scripts/test_http_api.sh

# Or run directly with cargo
cargo test --test http_api_test
```

### What Gets Tested

✅ **Health & Metrics**
- `/health` - Health check endpoint
- `/metrics` - Prometheus metrics

✅ **Time APIs**
- `/api/time` - Full UTC time details
- `/api/unix` - Unix timestamp
- `/api/nanos` - Nanosecond precision
- `/api/timezones` - List all IANA timezones
- `/api/time/timezone/:tz` - Time in specific timezone

✅ **NTP APIs**
- `/api/ntp/status` - NTP synchronization status

✅ **Additional Tests**
- CORS headers
- 404 error handling
- Concurrent request handling

### Test Output

```
🧪 Running HTTP API Integration Tests
======================================

running 13 tests
test test_health_endpoint ... ok
test test_api_time_endpoint ... ok
test test_api_unix_endpoint ... ok
test test_api_nanos_endpoint ... ok
test test_api_timezones_endpoint ... ok
test test_api_timezone_specific ... ok
test test_api_timezone_invalid ... ok
test test_api_ntp_status_container_mode ... ok
test test_metrics_endpoint ... ok
test test_404_endpoint ... ok
test test_cors_headers ... ok
test test_concurrent_requests ... ok

✅ All tests passed!
```

### Requirements

- Rust toolchain (any recent stable version)
- Port 13000 available (tests use this port)
- No Docker required

### Troubleshooting

**Port already in use:**
```bash
# Check what's using port 13000
lsof -i :13000

# Kill the process or change TEST_PORT in tests/http_api_test.rs
```

**Tests timing out:**
- Increase the sleep duration in tests
- Check system resources

## Manual Testing

You can also test the API manually:

```bash
# Start the server (in one terminal)
cargo run

# Test endpoints (in another terminal)
curl http://localhost:3000/health
curl http://localhost:3000/api/time
curl http://localhost:3000/api/unix
curl http://localhost:3000/api/timezones
curl http://localhost:3000/api/time/timezone/America/New_York
curl http://localhost:3000/api/ntp/status
curl http://localhost:3000/metrics
```

## CI/CD

These tests run automatically in GitHub Actions on every push:
- Tests run in parallel with other test suites
- Must pass before Docker image is built
- Ensures API compatibility
