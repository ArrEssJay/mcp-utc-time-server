#!/bin/bash
# Test comprehensive MCP protocol support
# Tests initialize, tools/list, tools/call, prompts/list, prompts/get

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

BINARY="${1:-target/release/mcp-utc-time-server}"
TEST_COUNT=0
PASS_COUNT=0
FAIL_COUNT=0

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}✗ Binary not found: $BINARY${NC}"
    echo "Usage: $0 [path-to-binary]"
    exit 1
fi

echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${BLUE}  MCP Protocol Compliance Test Suite${NC}"
echo -e "${BOLD}${BLUE}  Protocol Version: 2025-06-18${NC}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}\n"

# Helper function to send MCP request
send_request() {
    local method="$1"
    local params="${2:-{}}"
    local id="${3:-1}"
    
    # Send request and capture only JSON response (line 2, after the log line)
    echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":$id}" | \
        "$BINARY" 2>/dev/null | sed -n '1p' || echo '{"error": "no response"}'
}

# Test function
test_method() {
    local test_name="$1"
    local method="$2"
    local params="${3:-{}}"
    local check_field="$4"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    echo -e "${YELLOW}[$TEST_COUNT] Testing: $test_name${NC}"
    
    response=$(send_request "$method" "$params" "$TEST_COUNT")
    
    if echo "$response" | jq -e "$check_field" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ PASS${NC}"
        echo "$response" | jq '.' 2>/dev/null | head -20
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo -e "${RED}  ✗ FAIL${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    echo ""
}

# === LIFECYCLE TESTS ===
echo -e "${BOLD}${BLUE}=== Lifecycle Tests ===${NC}\n"

test_method \
    "initialize - Protocol version and capabilities" \
    "initialize" \
    '{"protocolVersion":"2025-06-18","clientInfo":{"name":"test-client","version":"1.0"}}' \
    '.result.protocolVersion'

test_method \
    "initialize - Server info" \
    "initialize" \
    '{"protocolVersion":"2025-06-18","clientInfo":{"name":"test-client","version":"1.0"}}' \
    '.result.serverInfo.name'

test_method \
    "initialize - Tools capability" \
    "initialize" \
    '{"protocolVersion":"2025-06-18","clientInfo":{"name":"test-client","version":"1.0"}}' \
    '.result.capabilities.tools'

test_method \
    "initialize - Prompts capability" \
    "initialize" \
    '{"protocolVersion":"2025-06-18","clientInfo":{"name":"test-client","version":"1.0"}}' \
    '.result.capabilities.prompts'

# === TOOLS TESTS ===
echo -e "${BOLD}${BLUE}=== Tools Tests ===${NC}\n"

test_method \
    "tools/list - Get all available tools" \
    "tools/list" \
    '{}' \
    '.result.tools | length > 0'

test_method \
    "tools/list - Verify tool structure (name, description, inputSchema)" \
    "tools/list" \
    '{}' \
    '.result.tools[0] | has("name") and has("description")'

test_method \
    "tools/call - get_time (no arguments)" \
    "tools/call" \
    '{"name":"get_time","arguments":{}}' \
    '.result.content[0].text'

test_method \
    "tools/call - get_unix_time" \
    "tools/call" \
    '{"name":"get_unix_time","arguments":{}}' \
    '.result.content[0].text'

test_method \
    "tools/call - get_time_formatted with format argument" \
    "tools/call" \
    '{"name":"get_time_formatted","arguments":{"format":"%Y-%m-%d %H:%M:%S"}}' \
    '.result.content[0].text'

test_method \
    "tools/call - get_time_with_timezone" \
    "tools/call" \
    '{"name":"get_time_with_timezone","arguments":{"timezone":"America/New_York"}}' \
    '.result.content[0].text'

test_method \
    "tools/call - list_timezones" \
    "tools/call" \
    '{"name":"list_timezones","arguments":{}}' \
    '.result.content[0].text'

test_method \
    "tools/call - Unknown tool returns error" \
    "tools/call" \
    '{"name":"nonexistent_tool","arguments":{}}' \
    '.result.isError == true'

# === PROMPTS TESTS ===
echo -e "${BOLD}${BLUE}=== Prompts Tests ===${NC}\n"

test_method \
    "prompts/list - Get all available prompts" \
    "prompts/list" \
    '{}' \
    '.result.prompts | length > 0'

test_method \
    "prompts/list - Verify prompt structure" \
    "prompts/list" \
    '{}' \
    '.result.prompts[0] | has("name") and has("description")'

test_method \
    "prompts/get - /time prompt (no arguments)" \
    "prompts/get" \
    '{"name":"time","arguments":{}}' \
    '.result.messages[0].content.text'

test_method \
    "prompts/get - /unix_time prompt" \
    "prompts/get" \
    '{"name":"unix_time","arguments":{}}' \
    '.result.messages[0].content.text'

test_method \
    "prompts/get - /time_in prompt with timezone" \
    "prompts/get" \
    '{"name":"time_in","arguments":{"timezone":"Europe/London"}}' \
    '.result.messages[0].content.text'

test_method \
    "prompts/get - /format_time prompt with format" \
    "prompts/get" \
    '{"name":"format_time","arguments":{"format":"%A, %B %d, %Y at %I:%M %p"}}' \
    '.result.messages[0].content.text'

# === LEGACY COMPATIBILITY TESTS ===
echo -e "${BOLD}${BLUE}=== Legacy Compatibility Tests ===${NC}\n"

test_method \
    "time/get - Legacy direct method" \
    "time/get" \
    '{}' \
    '.result.utc'

test_method \
    "time/get_unix - Legacy direct method" \
    "time/get_unix" \
    '{}' \
    '.result.seconds'

# === ERROR HANDLING TESTS ===
echo -e "${BOLD}${BLUE}=== Error Handling Tests ===${NC}\n"

test_method \
    "Unknown method returns error" \
    "unknown/method" \
    '{}' \
    '.error.code == -32601'

test_method \
    "tools/call - Missing name parameter" \
    "tools/call" \
    '{"arguments":{}}' \
    '.error'

test_method \
    "prompts/get - Missing name parameter" \
    "prompts/get" \
    '{"arguments":{}}' \
    '.error'

# === SUMMARY ===
echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${BLUE}  Test Summary${NC}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Total Tests:${NC}  $TEST_COUNT"
echo -e "${GREEN}${BOLD}Passed:${NC}       $PASS_COUNT"
echo -e "${RED}${BOLD}Failed:${NC}       $FAIL_COUNT"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}\n"

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}${BOLD}✓ All MCP protocol tests passed!${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}✗ Some tests failed. Please review the output above.${NC}"
    exit 1
fi
