#!/usr/bin/env bash
# Example script demonstrating MCP UTC Time Server usage

set -euo pipefail

SERVER="cargo run --release 2>/dev/null"

echo "=== MCP UTC Time Server Examples ==="
echo

echo "1. Get complete time information:"
echo '{"jsonrpc":"2.0","method":"time/get","params":{},"id":1}' | $SERVER | jq '.result | {unix: .unix.seconds, iso8601, timezone}'
echo

echo "2. Get Unix timestamp with nanosecond precision:"
echo '{"jsonrpc":"2.0","method":"time/get_unix","params":{},"id":2}' | $SERVER | jq '.result'
echo

echo "3. Get nanoseconds since epoch:"
echo '{"jsonrpc":"2.0","method":"time/get_nanos","params":{},"id":3}' | $SERVER | jq '.result.nanoseconds'
echo

echo "4. Custom format (strftime):"
echo '{"jsonrpc":"2.0","method":"time/get_with_format","params":{"format":"%Y-%m-%d %H:%M:%S"},"id":4}' | $SERVER | jq -r '.result.formatted'
echo

echo "5. Get time in Tokyo timezone:"
echo '{"jsonrpc":"2.0","method":"time/get_with_timezone","params":{"timezone":"Asia/Tokyo"},"id":5}' | $SERVER | jq '.result | {timezone, offset, iso8601}'
echo

echo "6. Convert timestamp between timezones:"
echo '{"jsonrpc":"2.0","method":"time/convert","params":{"timestamp":1760558400,"from_timezone":"UTC","to_timezone":"Europe/London"},"id":6}' | $SERVER | jq '.result'
echo

echo "=== All tests completed successfully ==="
