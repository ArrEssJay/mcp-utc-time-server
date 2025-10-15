#!/usr/bin/env bash
# Test VSCode MCP integration

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$PROJECT_DIR/target/release/mcp-utc-time-server"

echo "=== VSCode MCP Integration Test ==="
echo

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "Building release binary..."
    cd "$PROJECT_DIR"
    cargo build --release
    echo
fi

# Test 1: Binary runs
echo "1. Testing binary execution:"
if [ -x "$BINARY" ]; then
    echo "✓ Binary is executable"
else
    echo "✗ Binary is not executable"
    chmod +x "$BINARY"
    echo "  Fixed: Made binary executable"
fi

# Test 2: STDIO communication
echo
echo "2. Testing STDIO communication:"
result=$(echo '{"jsonrpc":"2.0","method":"time/get","params":{},"id":1}' | "$BINARY" 2>/dev/null | head -1)
if echo "$result" | jq -e '.result.unix.seconds' > /dev/null 2>&1; then
    echo "✓ Server responds to JSON-RPC requests"
else
    echo "✗ Server did not respond correctly"
fi

# Test 3: Initialize method
echo
echo "3. Testing initialize method:"
result=$(echo '{"jsonrpc":"2.0","method":"initialize","params":{"clientInfo":{"name":"vscode","version":"1.0.0"},"protocolVersion":"2024-11-05"},"id":1}' | "$BINARY" 2>/dev/null | head -1)
if echo "$result" | jq -e '.result.capabilities.tools' > /dev/null 2>&1; then
    echo "✓ Initialize handshake works"
else
    echo "✗ Initialize failed"
fi

# Test 4: All methods
echo
echo "4. Testing all MCP methods:"
methods=("time/get" "time/get_unix" "time/get_nanos" "time/list_timezones")
for method in "${methods[@]}"; do
    result=$(echo '{"jsonrpc":"2.0","method":"'$method'","params":{},"id":1}' | "$BINARY" 2>/dev/null | head -1)
    if echo "$result" | jq -e '.result' > /dev/null 2>&1; then
        echo "  ✓ $method"
    else
        echo "  ✗ $method"
    fi
done

# Test 5: Custom format method
result=$(echo '{"jsonrpc":"2.0","method":"time/get_with_format","params":{"format":"%Y-%m-%d"},"id":1}' | "$BINARY" 2>/dev/null | head -1)
if echo "$result" | jq -e '.result.formatted' > /dev/null 2>&1; then
    echo "  ✓ time/get_with_format"
else
    echo "  ✗ time/get_with_format"
fi

# Test 6: Timezone method
result=$(echo '{"jsonrpc":"2.0","method":"time/get_with_timezone","params":{"timezone":"Asia/Tokyo"},"id":1}' | "$BINARY" 2>/dev/null | head -1)
if echo "$result" | jq -e '.result.timezone' > /dev/null 2>&1; then
    echo "  ✓ time/get_with_timezone"
else
    echo "  ✗ time/get_with_timezone"
fi

# Test 7: Convert method
result=$(echo '{"jsonrpc":"2.0","method":"time/convert","params":{"timestamp":1700000000,"to_timezone":"America/New_York"},"id":1}' | "$BINARY" 2>/dev/null | head -1)
if echo "$result" | jq -e '.result.converted' > /dev/null 2>&1; then
    echo "  ✓ time/convert"
else
    echo "  ✗ time/convert"
fi

echo
echo "=== VSCode Configuration ==="
echo
echo "Add this to your .vscode/settings.json:"
echo
cat << 'EOF'
{
    "mcp.servers": {
        "utc-time": {
            "command": "FULL_PATH_TO/target/release/mcp-utc-time-server",
            "args": [],
            "env": {
                "RUST_LOG": "info",
                "TZ": "UTC"
            }
        }
    }
}
EOF
echo
echo "Replace FULL_PATH_TO with: $PROJECT_DIR"
echo
echo "=== Test Complete ==="
