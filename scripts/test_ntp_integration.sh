#!/bin/bash
# Test NTP integration with MCP server

set -e

echo "=== MCP UTC Time Server - NTP Integration Test ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((TESTS_FAILED++))
    ((TESTS_RUN++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Check if binary exists
if [ ! -f "target/release/mcp-utc-time-server" ]; then
    info "Building release binary..."
    cargo build --release
fi

# Test 1: Health endpoint
info "Testing HTTP health endpoint..."
timeout 5s cargo run --release > /dev/null 2>&1 &
PID=$!
sleep 2

if curl -s http://localhost:3000/health | jq -e '.status == "healthy"' > /dev/null 2>&1; then
    pass "Health endpoint responds correctly"
else
    fail "Health endpoint not responding"
fi

# Test 2: Metrics endpoint
if curl -s http://localhost:3000/metrics | grep -q "mcp_time_seconds"; then
    pass "Metrics endpoint provides Prometheus metrics"
else
    fail "Metrics endpoint not working"
fi

kill $PID 2>/dev/null || true
sleep 1

# Test 3: NTP status tool via STDIO
info "Testing NTP tools via MCP protocol..."

# Create test request for get_ntp_status
cat > /tmp/mcp_test_ntp.json <<EOF
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_ntp_status","arguments":{}}}
EOF

# Run server with test input
RESULT=$(echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_ntp_status","arguments":{}}}' | timeout 5s cargo run --release 2>/dev/null | grep -A 20 '"result"' | head -20)

if echo "$RESULT" | grep -q "available"; then
    pass "NTP status tool responds"
else
    fail "NTP status tool not responding"
fi

# Test 4: NTP peers tool
RESULT=$(echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_ntp_peers","arguments":{}}}' | timeout 5s cargo run --release 2>/dev/null | grep -A 20 '"result"' | head -20)

if echo "$RESULT" | grep -q "available"; then
    pass "NTP peers tool responds"
else
    fail "NTP peers tool not responding"
fi

# Test 5: Time tools still work
RESULT=$(echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_time","arguments":{}}}' | timeout 5s cargo run --release 2>/dev/null | grep -A 20 '"result"' | head -20)

if echo "$RESULT" | grep -q "unix"; then
    pass "Time tools still functional"
else
    fail "Time tools broken"
fi

# Test 6: Server info includes NTP tools
RESULT=$(echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | timeout 5s cargo run --release 2>/dev/null | grep -A 50 '"result"')

if echo "$RESULT" | grep -q "get_ntp_status"; then
    pass "NTP status tool listed in tools"
else
    fail "NTP status tool not in tools list"
fi

if echo "$RESULT" | grep -q "get_ntp_peers"; then
    pass "NTP peers tool listed in tools"
else
    fail "NTP peers tool not in tools list"
fi

# Summary
echo ""
echo "=== Test Summary ==="
echo "Tests run: $TESTS_RUN"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    exit 1
else
    echo "All tests passed! ✨"
    exit 0
fi
