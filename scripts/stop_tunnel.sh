#!/usr/bin/env bash
# Stop MCP UTC Time Server tunnel and MCPO server

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TUNNEL_URL_FILE="${PROJECT_DIR}/.tunnel_url"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Stopping MCP UTC Time Server tunnel...${NC}"
echo

STOPPED_SOMETHING=0

# Stop cloudflared processes
if pgrep -f "cloudflared tunnel" > /dev/null; then
    echo "Stopping cloudflared tunnel..."
    pkill -f "cloudflared tunnel" || true
    STOPPED_SOMETHING=1
    echo -e "${GREEN}✓ Cloudflared stopped${NC}"
else
    echo -e "${YELLOW}No cloudflared tunnel running${NC}"
fi

# Stop MCPO processes
if pgrep -f "mcpo.*mcp-utc-time-server" > /dev/null; then
    echo "Stopping MCPO server..."
    pkill -f "mcpo.*mcp-utc-time-server" || true
    STOPPED_SOMETHING=1
    echo -e "${GREEN}✓ MCPO stopped${NC}"
else
    echo -e "${YELLOW}No MCPO server running${NC}"
fi

# Clean up tunnel URL file
if [ -f "$TUNNEL_URL_FILE" ]; then
    rm -f "$TUNNEL_URL_FILE"
    echo -e "${GREEN}✓ Tunnel URL file removed${NC}"
fi

# Clean up log files (optional)
if [ -f "$PROJECT_DIR/tunnel.log" ] || [ -f "$PROJECT_DIR/mcpo.log" ]; then
    echo
    read -p "Remove log files? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f "$PROJECT_DIR/tunnel.log" "$PROJECT_DIR/mcpo.log"
        echo -e "${GREEN}✓ Log files removed${NC}"
    fi
fi

echo

if [ $STOPPED_SOMETHING -eq 1 ]; then
    echo -e "${GREEN}✓ Tunnel and server stopped successfully${NC}"
else
    echo -e "${YELLOW}Nothing was running${NC}"
fi

# Verify nothing is running
if pgrep -f "cloudflared tunnel" > /dev/null || pgrep -f "mcpo.*mcp-utc-time-server" > /dev/null; then
    echo
    echo -e "${RED}Warning: Some processes may still be running${NC}"
    echo
    echo "Check with:"
    echo -e "  ${GREEN}ps aux | grep cloudflared${NC}"
    echo -e "  ${GREEN}ps aux | grep mcpo${NC}"
    echo
    echo "Force kill with:"
    echo -e "  ${GREEN}pkill -9 cloudflared${NC}"
    echo -e "  ${GREEN}pkill -9 mcpo${NC}"
fi
