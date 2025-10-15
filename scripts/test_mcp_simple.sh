#!/bin/bash
# Simple MCP Protocol Test
# Tests key MCP methods per 2025-06-18 spec

BINARY="${1:-target/release/mcp-utc-time-server}"

if [ ! -f "$BINARY" ]; then
    echo "✗ Binary not found: $BINARY"
    exit 1
fi

echo "═══════════════════════════════════════════════════════"
echo "  MCP Protocol Test (2025-06-18)"
echo "═══════════════════════════════════════════════════════"
echo ""

test_count=0
pass_count=0

# Helper to run test
run_test() {
    test_count=$((test_count + 1))
    local name="$1"
    local json="$2"
    local check="$3"
    
    echo "[$test_count] $name"
    result=$(echo "$json" | "$BINARY" 2>/dev/null | head -1)
    
    if echo "$result" | jq -e "$check" >/dev/null 2>&1; then
        echo "  ✓ PASS"
        pass_count=$((pass_count + 1))
    else
        echo "  ✗ FAIL"
        echo "$result" | jq '.' 2>/dev/null || echo "$result"
    fi
    echo ""
}

# === CORE PROTOCOL ===
echo "=== Core Protocol ==="
echo ""

run_test "initialize" \
    '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
    '.result.protocolVersion == "2025-06-18"'

run_test "initialize capabilities" \
    '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test","version":"1.0"}},"id":2}' \
    '.result.capabilities | has("tools") and has("prompts")'

# === TOOLS ===
echo "=== Tools ==="
echo ""

run_test "tools/list" \
    '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}' \
    '.result.tools | length == 7'

run_test "tools/call - get_time" \
    '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_time","arguments":{}},"id":4}' \
    '.result.content[0].text'

run_test "tools/call - get_time_formatted" \
    '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_time_formatted","arguments":{"format":"%Y-%m-%d"}},"id":5}' \
    '.result.content[0].text'

run_test "tools/call - get_time_with_timezone" \
    '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_time_with_timezone","arguments":{"timezone":"America/New_York"}},"id":6}' \
    '.result.content[0].text'

# === PROMPTS ===
echo "=== Prompts ===" 
echo ""

run_test "prompts/list" \
    '{"jsonrpc":"2.0","method":"prompts/list","params":{},"id":7}' \
    '.result.prompts | length == 4'

run_test "prompts/get - time" \
    '{"jsonrpc":"2.0","method":"prompts/get","params":{"name":"time","arguments":{}},"id":8}' \
    '.result.messages[0].content.text'

run_test "prompts/get - time_in" \
    '{"jsonrpc":"2.0","method":"prompts/get","params":{"name":"time_in","arguments":{"timezone":"Europe/London"}},"id":9}' \
    '.result.messages[0].content.text'

run_test "prompts/get - format_time" \
    '{"jsonrpc":"2.0","method":"prompts/get","params":{"name":"format_time","arguments":{"format":"%A, %B %d"}},"id":10}' \
    '.result.messages[0].content.text'

# === LEGACY COMPATIBILITY ===
echo "=== Legacy Methods ==="
echo ""

run_test "time/get (legacy)" \
    '{"jsonrpc":"2.0","method":"time/get","params":{},"id":11}' \
    '.result.unix.seconds'

run_test "time/get_unix (legacy)" \
    '{"jsonrpc":"2.0","method":"time/get_unix","params":{},"id":12}' \
    '.result.seconds'

# === SUMMARY ===
echo "═══════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════"
echo "Total:  $test_count"
echo "Passed: $pass_count"
echo "Failed: $((test_count - pass_count))"
echo "═══════════════════════════════════════════════════════"
echo ""

if [ $pass_count -eq $test_count ]; then
    echo "✓ All MCP protocol tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi
