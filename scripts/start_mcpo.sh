#!/usr/bin/env bash
# Start MCP UTC Time Server with MCPO wrapper

set -euo pipefail

# Configuration
PORT="${MCPO_PORT:-8000}"
API_KEY="${MCPO_API_KEY}"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== MCP UTC Time Server with MCPO ===${NC}"
echo

# Check if API key is set
if [ -z "$API_KEY" ]; then
    echo -e "${RED}ERROR: MCPO_API_KEY environment variable is not set${NC}"
    echo
    echo "Generate a secure API key:"
    echo "  export MCPO_API_KEY=\$(openssl rand -base64 32)"
    echo
    echo "Or set a custom key:"
    echo "  export MCPO_API_KEY='your-secret-key'"
    echo
    exit 1
fi

# Check if mcpo is installed
if ! command -v mcpo &> /dev/null && ! command -v uvx &> /dev/null; then
    echo -e "${RED}ERROR: mcpo not found${NC}"
    echo
    echo "Install mcpo using:"
    echo "  pip install mcpo"
    echo "  # or"
    echo "  pipx install mcpo"
    exit 1
fi

# Build release binary if it doesn't exist
if [ ! -f "$PROJECT_DIR/target/release/mcp-utc-time-server" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release
    echo -e "${GREEN}Build complete!${NC}"
    echo
fi

# Display configuration
echo "Configuration:"
echo "  Port: $PORT"
echo "  API Key: ${API_KEY:0:10}... (hidden)"
echo "  Binary: $PROJECT_DIR/target/release/mcp-utc-time-server"
echo

# Start MCPO server
echo -e "${GREEN}Starting MCPO server...${NC}"
echo "Press Ctrl+C to stop"
echo

if command -v uvx &> /dev/null; then
    uvx mcpo --port "$PORT" --api-key "$API_KEY" -- \
        "$PROJECT_DIR/target/release/mcp-utc-time-server"
elif command -v mcpo &> /dev/null; then
    mcpo --port "$PORT" --api-key "$API_KEY" -- \
        "$PROJECT_DIR/target/release/mcp-utc-time-server"
else
    echo -e "${RED}ERROR: Neither uvx nor mcpo command found${NC}"
    exit 1
fi
