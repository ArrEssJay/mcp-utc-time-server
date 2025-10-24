#!/bin/bash
# Test get_time_with_timezone via MCP STDIO

set -e

BINARY="target/release/mcp-utc-time-server"

# Create a named pipe for bidirectional communication
FIFO="/tmp/mcp_test_$$"
mkfifo "$FIFO"

# Start the server in background, reading from FIFO
"$BINARY" < "$FIFO" 2>/dev/null &
SERVER_PID=$!

# Give server time to start
sleep 0.2

# Open FIFO for writing
exec 3>"$FIFO"

# Function to send request and wait for response
send_request() {
    echo "$1" >&3
    sleep 0.1
}

# 1. Initialize
echo "Sending initialize request..."
send_request '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test-client","version":"1.0"},"capabilities":{}},"id":1}'

# 2. Send initialized notification
echo "Sending initialized notification..."
send_request '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'

# 3. Call get_time_with_timezone
echo "Calling get_time_with_timezone for America/New_York..."
send_request '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_time_with_timezone","arguments":{"timezone":"America/New_York"}},"id":2}'

# Wait a bit for response
sleep 0.5

# Cleanup
exec 3>&-
kill $SERVER_PID 2>/dev/null || true
rm -f "$FIFO"

echo "Done!"
