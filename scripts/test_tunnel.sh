#!/usr/bin/env bash
# Test Cloudflare Tunnel connectivity and API functionality

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TUNNEL_URL_FILE="${PROJECT_DIR}/.tunnel_url"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Check if tunnel URL file exists
if [ ! -f "$TUNNEL_URL_FILE" ]; then
    echo -e "${RED}ERROR: Tunnel URL file not found${NC}"
    echo
    echo "Start the tunnel first:"
    echo -e "  ${GREEN}./scripts/start_tunnel.sh${NC}"
    echo
    exit 1
fi

# Read tunnel URL
TUNNEL_URL=$(cat "$TUNNEL_URL_FILE")
API_KEY="${MCPO_API_KEY:-}"

if [ -z "$API_KEY" ]; then
    echo -e "${RED}ERROR: MCPO_API_KEY environment variable not set${NC}"
    echo
    echo "Set your API key:"
    echo -e "  ${GREEN}export MCPO_API_KEY='your-key'${NC}"
    echo
    exit 1
fi

echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        Cloudflare Tunnel Connectivity Test        ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
echo
echo -e "${BLUE}Tunnel URL:${NC} $TUNNEL_URL"
echo -e "${BLUE}API Key:${NC} ${API_KEY:0:16}... (hidden)"
echo
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_code="${3:-0}"
    
    echo -e "${BLUE}Testing: $test_name${NC}"
    
    if eval "$test_command" > /tmp/test_output 2>&1; then
        actual_code=0
    else
        actual_code=$?
    fi
    
    if [ "$actual_code" -eq "$expected_code" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        
        # Show response if JSON
        if command -v jq &> /dev/null && jq empty < /tmp/test_output 2>/dev/null; then
            echo "  Response:"
            jq '.' < /tmp/test_output | sed 's/^/    /'
        fi
    else
        echo -e "${RED}✗ FAIL (exit code: $actual_code, expected: $expected_code)${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo "  Output:"
        cat /tmp/test_output | sed 's/^/    /'
    fi
    
    echo
}

# Run tests
echo -e "${YELLOW}Running connectivity tests...${NC}"
echo

# Test 1: Health check (no auth required)
run_test "1. Health Check" \
    "curl -s -f -m 10 '$TUNNEL_URL/health'"

# Test 2: OpenAPI schema (no auth required)
run_test "2. OpenAPI Schema" \
    "curl -s -f -m 10 '$TUNNEL_URL/openapi.json' | jq -e '.openapi'"

# Test 3: Unauthorized request (should fail)
run_test "3. Unauthorized Access (should fail)" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get' -H 'Content-Type: application/json'" 22

# Test 4: Get current time
run_test "4. Get Current Time" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' | jq -e '.result.unix.seconds'"

# Test 5: Get Unix timestamp
run_test "5. Get Unix Timestamp" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get_unix' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' | jq -e '.result.seconds'"

# Test 6: Get nanoseconds
run_test "6. Get Nanoseconds" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get_nanos' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' | jq -e '.result.nanos_since_epoch'"

# Test 7: Custom format
run_test "7. Custom Format (strftime)" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get_with_format' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' \
        -d '{\"format\": \"%Y-%m-%d %H:%M:%S\"}' | jq -e '.result.formatted'"

# Test 8: Timezone conversion
run_test "8. Timezone Conversion" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/get_with_timezone' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' \
        -d '{\"timezone\": \"Asia/Tokyo\"}' | jq -e '.result.timezone'"

# Test 9: List timezones
run_test "9. List Timezones" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/list_timezones' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' | jq -e '.result.timezones | length'"

# Test 10: Convert timezone
run_test "10. Convert Timezone" \
    "curl -s -f -m 10 -X POST '$TUNNEL_URL/time/convert' \
        -H 'Authorization: Bearer $API_KEY' \
        -H 'Content-Type: application/json' \
        -d '{\"from_tz\": \"UTC\", \"to_tz\": \"America/New_York\"}' | jq -e '.result.converted_time'"

# Display results
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Test Results:${NC}"
echo -e "  ${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "  ${RED}Failed: $TESTS_FAILED${NC}"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed! Tunnel is working correctly.${NC}"
    echo
    echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Next Steps:${NC}"
    echo
    echo "1. Configure ChatGPT Custom GPT:"
    echo -e "   ${GREEN}https://chat.openai.com/gpts/editor${NC}"
    echo
    echo "2. Import OpenAPI schema from:"
    echo -e "   ${GREEN}$TUNNEL_URL/openapi.json${NC}"
    echo
    echo "3. Set Bearer token authentication:"
    echo -e "   ${YELLOW}$API_KEY${NC}"
    echo
    echo "4. Add servers array to imported JSON:"
    echo '   "servers": ['
    echo '     {'
    echo -e "       \"url\": \"${GREEN}$TUNNEL_URL${NC}\""
    echo '     }'
    echo '   ],'
    echo
    exit 0
else
    echo -e "${RED}✗ Some tests failed. Check the output above.${NC}"
    echo
    echo "Troubleshooting:"
    echo "1. Verify tunnel is running:"
    echo -e "   ${GREEN}ps aux | grep cloudflared${NC}"
    echo
    echo "2. Check tunnel logs:"
    echo -e "   ${GREEN}cat $PROJECT_DIR/tunnel.log${NC}"
    echo
    echo "3. Test local endpoint:"
    echo -e "   ${GREEN}curl http://localhost:8000/health${NC}"
    echo
    exit 1
fi
