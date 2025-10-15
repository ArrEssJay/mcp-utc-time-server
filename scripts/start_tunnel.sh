#!/usr/bin/env bash
# Start MCP UTC Time Server with MCPO and Cloudflare Tunnel
# This script manages both the MCPO HTTP wrapper and cloudflared tunnel

set -euo pipefail

# Configuration
PORT="${MCPO_PORT:-8000}"
API_KEY="${MCPO_API_KEY}"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TUNNEL_LOG="${PROJECT_DIR}/tunnel.log"
MCPO_LOG="${PROJECT_DIR}/mcpo.log"
TUNNEL_URL_FILE="${PROJECT_DIR}/.tunnel_url"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}Shutting down...${NC}"
    
    # Kill MCPO if running
    if [ -n "${MCPO_PID:-}" ] && kill -0 "$MCPO_PID" 2>/dev/null; then
        echo "Stopping MCPO server (PID: $MCPO_PID)..."
        kill "$MCPO_PID" 2>/dev/null || true
        wait "$MCPO_PID" 2>/dev/null || true
    fi
    
    # Kill cloudflared if running
    if [ -n "${TUNNEL_PID:-}" ] && kill -0 "$TUNNEL_PID" 2>/dev/null; then
        echo "Stopping Cloudflare tunnel (PID: $TUNNEL_PID)..."
        kill "$TUNNEL_PID" 2>/dev/null || true
        wait "$TUNNEL_PID" 2>/dev/null || true
    fi
    
    # Clean up tunnel URL file
    rm -f "$TUNNEL_URL_FILE"
    
    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT INT TERM

echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  MCP UTC Time Server - Cloudflare Tunnel Setup   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
echo

# Check if API key is set
if [ -z "${API_KEY:-}" ]; then
    echo -e "${RED}ERROR: MCPO_API_KEY environment variable is not set${NC}"
    echo
    echo "Generate a secure API key:"
    echo -e "  ${GREEN}export MCPO_API_KEY=\"\$(openssl rand -base64 32)\"${NC}"
    echo
    echo "Or set a custom key:"
    echo -e "  ${GREEN}export MCPO_API_KEY='your-secret-key'${NC}"
    echo
    echo "For testing only (insecure):"
    echo -e "  ${YELLOW}export MCPO_API_KEY='test-key-do-not-use-in-production'${NC}"
    echo
    exit 1
fi

# Check if mcpo is installed
if ! command -v mcpo &> /dev/null && ! command -v uvx &> /dev/null; then
    echo -e "${RED}ERROR: mcpo not found${NC}"
    echo
    echo "Install mcpo using:"
    echo -e "  ${GREEN}pip install mcpo${NC}"
    echo "  ${GREEN}# or${NC}"
    echo -e "  ${GREEN}pipx install mcpo${NC}"
    echo
    exit 1
fi

# Check if cloudflared is installed
if ! command -v cloudflared &> /dev/null; then
    echo -e "${RED}ERROR: cloudflared not found${NC}"
    echo
    echo "Install cloudflared using:"
    echo -e "  ${GREEN}brew install cloudflare/cloudflare/cloudflared${NC}"
    echo
    echo "Or download from:"
    echo "  https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/"
    echo
    exit 1
fi

# Check if port is available
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${RED}ERROR: Port $PORT is already in use${NC}"
    echo
    echo "Find the process using the port:"
    echo -e "  ${GREEN}lsof -i :$PORT${NC}"
    echo
    echo "Or use a different port:"
    echo -e "  ${GREEN}export MCPO_PORT=8001${NC}"
    echo
    exit 1
fi

# Build release binary if it doesn't exist
if [ ! -f "$PROJECT_DIR/target/release/mcp-utc-time-server" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release
    echo -e "${GREEN}✓ Build complete!${NC}"
    echo
fi

# Display configuration
echo -e "${BLUE}Configuration:${NC}"
echo "  Port:        $PORT"
echo "  API Key:     ${API_KEY:0:16}... (hidden)"
echo "  Binary:      $PROJECT_DIR/target/release/mcp-utc-time-server"
echo "  MCPO Log:    $MCPO_LOG"
echo "  Tunnel Log:  $TUNNEL_LOG"
echo

# Start MCPO server in background
echo -e "${GREEN}Starting MCPO server...${NC}"

MCPO_CMD=""
if command -v uvx &> /dev/null; then
    MCPO_CMD="uvx mcpo --port $PORT --api-key $API_KEY -- $PROJECT_DIR/target/release/mcp-utc-time-server"
else
    MCPO_CMD="mcpo --port $PORT --api-key $API_KEY -- $PROJECT_DIR/target/release/mcp-utc-time-server"
fi

# Start MCPO in background
$MCPO_CMD > "$MCPO_LOG" 2>&1 &
MCPO_PID=$!

# Wait for MCPO to start
echo "Waiting for MCPO server to start..."
sleep 2

# Check if MCPO is running
if ! kill -0 "$MCPO_PID" 2>/dev/null; then
    echo -e "${RED}ERROR: MCPO server failed to start${NC}"
    echo
    echo "Check logs:"
    echo -e "  ${GREEN}cat $MCPO_LOG${NC}"
    exit 1
fi

echo -e "${GREEN}✓ MCPO server started (PID: $MCPO_PID)${NC}"

# Test local endpoint
echo "Testing local endpoint..."
if curl -s -f "http://localhost:$PORT/health" > /dev/null; then
    echo -e "${GREEN}✓ Local endpoint is healthy${NC}"
else
    echo -e "${YELLOW}⚠ Health check failed, but continuing...${NC}"
fi

echo

# Start Cloudflare tunnel
echo -e "${GREEN}Starting Cloudflare tunnel...${NC}"
echo "This may take 10-15 seconds..."

cloudflared tunnel --url "http://localhost:$PORT" > "$TUNNEL_LOG" 2>&1 &
TUNNEL_PID=$!

# Wait for tunnel URL
echo "Waiting for tunnel URL..."
TUNNEL_URL=""
ATTEMPTS=0
MAX_ATTEMPTS=30

while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    if [ -f "$TUNNEL_LOG" ]; then
        TUNNEL_URL=$(grep -oE 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' "$TUNNEL_LOG" | head -n 1 || true)
        if [ -n "$TUNNEL_URL" ]; then
            break
        fi
    fi
    sleep 1
    ATTEMPTS=$((ATTEMPTS + 1))
    echo -n "."
done

echo

if [ -z "$TUNNEL_URL" ]; then
    echo -e "${RED}ERROR: Failed to get tunnel URL${NC}"
    echo
    echo "Check tunnel log:"
    echo -e "  ${GREEN}cat $TUNNEL_LOG${NC}"
    exit 1
fi

# Save tunnel URL
echo "$TUNNEL_URL" > "$TUNNEL_URL_FILE"

echo -e "${GREEN}✓ Cloudflare tunnel established${NC}"
echo

# Display tunnel information
echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              Tunnel Active & Ready!               ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
echo
echo -e "${BLUE}Public URL:${NC}"
echo -e "  ${GREEN}$TUNNEL_URL${NC}"
echo
echo -e "${BLUE}API Key:${NC}"
echo -e "  ${YELLOW}$API_KEY${NC}"
echo
echo -e "${BLUE}OpenAPI Schema:${NC}"
echo -e "  ${GREEN}$TUNNEL_URL/openapi.json${NC}"
echo
echo -e "${BLUE}Health Check:${NC}"
echo -e "  ${GREEN}$TUNNEL_URL/health${NC}"
echo

# Test tunnel endpoint
echo -e "${BLUE}Testing tunnel endpoint...${NC}"
if curl -s -f "$TUNNEL_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Tunnel endpoint is accessible${NC}"
else
    echo -e "${YELLOW}⚠ Tunnel health check failed (may need more time)${NC}"
fi

echo

# Display usage examples
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Quick Test Commands:${NC}"
echo
echo "1. Health check:"
echo -e "   ${GREEN}curl $TUNNEL_URL/health${NC}"
echo
echo "2. Get current time:"
echo -e "   ${GREEN}curl -X POST $TUNNEL_URL/time/get \\${NC}"
echo -e "   ${GREEN}  -H \"Authorization: Bearer $API_KEY\" \\${NC}"
echo -e "   ${GREEN}  -H \"Content-Type: application/json\"${NC}"
echo
echo "3. Get OpenAPI schema:"
echo -e "   ${GREEN}curl $TUNNEL_URL/openapi.json | jq .${NC}"
echo

echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}ChatGPT Custom GPT Setup:${NC}"
echo
echo "1. Go to: https://chat.openai.com/gpts/editor"
echo "2. Configure → Actions → Add Action"
echo "3. Authentication:"
echo "   - Type: API Key"
echo "   - Auth Type: Bearer"
echo -e "   - API Key: ${YELLOW}$API_KEY${NC}"
echo
echo "4. Import OpenAPI Schema:"
echo "   - Click 'Import from URL'"
echo -e "   - Enter: ${GREEN}$TUNNEL_URL/openapi.json${NC}"
echo
echo "5. Edit the imported JSON - add servers array after 'openapi' field:"
echo '   "servers": ['
echo '     {'
echo -e "       \"url\": \"${GREEN}$TUNNEL_URL${NC}\""
echo '     }'
echo '   ],'
echo
echo "6. Save and test!"
echo

echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Server is running. Press Ctrl+C to stop.${NC}"
echo
echo "Monitoring logs:"
echo -e "  MCPO:   ${GREEN}tail -f $MCPO_LOG${NC}"
echo -e "  Tunnel: ${GREEN}tail -f $TUNNEL_LOG${NC}"
echo

# Keep script running and display live logs
echo -e "${BLUE}Live MCPO logs (Ctrl+C to stop):${NC}"
echo "─────────────────────────────────────────────────────"

# Tail MCPO logs until interrupted
tail -f "$MCPO_LOG" 2>/dev/null || true
