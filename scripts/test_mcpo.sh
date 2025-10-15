#!/usr/bin/env bash
# Test script for MCPO integration

set -euo pipefail

API_KEY="${MCPO_API_KEY:-test-key-change-me}"
BASE_URL="${MCPO_URL:-http://localhost:8000}"

echo "=== MCPO Integration Test Suite ==="
echo "Base URL: $BASE_URL"
echo

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "Warning: jq not found. Install for better output formatting."
    echo "  macOS: brew install jq"
    echo "  Linux: apt-get install jq"
    echo
fi

# Test 1: Health Check
echo "1. Health Check:"
if command -v jq &> /dev/null; then
    curl -s "$BASE_URL/health" | jq '.' || echo "FAILED"
else
    curl -s "$BASE_URL/health" || echo "FAILED"
fi
echo

# Test 2: OpenAPI Schema
echo "2. OpenAPI Schema:"
if command -v jq &> /dev/null; then
    curl -s "$BASE_URL/openapi.json" | jq '.info, .servers' || echo "FAILED"
else
    curl -s "$BASE_URL/openapi.json" | head -20 || echo "FAILED"
fi
echo

# Test 3: Get UTC Time (with auth)
echo "3. Get Complete UTC Time:"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" | jq '.result | {unix: .unix.seconds, iso8601, timezone, nanos_since_epoch}' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/get" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" || echo "FAILED"
fi
echo

# Test 4: Get Unix Timestamp
echo "4. Get Unix Timestamp:"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get_unix" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" | jq '.result' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/get_unix" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" || echo "FAILED"
fi
echo

# Test 5: Get Nanoseconds
echo "5. Get Nanoseconds Since Epoch:"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get_nanos" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" | jq '.result.nanoseconds' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/get_nanos" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" || echo "FAILED"
fi
echo

# Test 6: Custom Format
echo "6. Custom Format (strftime):"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get_with_format" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"format": "%Y-%m-%d %H:%M:%S %Z"}' | jq -r '.result.formatted' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/get_with_format" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"format": "%Y-%m-%d %H:%M:%S %Z"}' || echo "FAILED"
fi
echo

# Test 7: Timezone Conversion
echo "7. Get Time in Tokyo:"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get_with_timezone" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timezone": "Asia/Tokyo"}' | jq '.result | {timezone, offset, iso8601}' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/get_with_timezone" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timezone": "Asia/Tokyo"}' || echo "FAILED"
fi
echo

# Test 8: List Timezones (limited output)
echo "8. List Timezones (first 5):"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/list_timezones" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" | jq '.result.timezones[:5]' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/list_timezones" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" || echo "FAILED"
fi
echo

# Test 9: Convert Timestamp
echo "9. Convert Timestamp Between Timezones:"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/convert" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timestamp": 1700000000, "from_timezone": "UTC", "to_timezone": "America/New_York"}' | jq '.result' || echo "FAILED"
else
    curl -s -X POST "$BASE_URL/time/convert" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timestamp": 1700000000, "from_timezone": "UTC", "to_timezone": "America/New_York"}' || echo "FAILED"
fi
echo

# Test 10: Error handling (invalid timezone)
echo "10. Error Handling (Invalid Timezone):"
if command -v jq &> /dev/null; then
    curl -s -X POST "$BASE_URL/time/get_with_timezone" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timezone": "Invalid/Timezone"}' | jq '.error' || echo "PASSED (expected error)"
else
    curl -s -X POST "$BASE_URL/time/get_with_timezone" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"timezone": "Invalid/Timezone"}' || echo "PASSED (expected error)"
fi
echo

echo "=== All Tests Completed ==="
echo
echo "Summary:"
echo "- If you see 'FAILED', check that MCPO server is running on $BASE_URL"
echo "- If you see 401 errors, verify MCPO_API_KEY environment variable"
echo "- For detailed logs, run: RUST_LOG=debug cargo run --release"
