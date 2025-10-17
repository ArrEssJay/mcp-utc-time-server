#!/bin/sh
# Health check script for MCP Time Server

set -e

# Check if NTP daemon is running
if ! pgrep -x ntpd > /dev/null; then
    echo "ERROR: NTPd not running"
    exit 1
fi

# Check if MCP server is running and responding
if ! curl -f -s -m 2 http://localhost:${PORT:-8080}/health > /dev/null 2>&1; then
    echo "ERROR: MCP server not responding"
    exit 1
fi

# Check NTP synchronization status
if command -v ntpq >/dev/null 2>&1; then
    if ! ntpq -p > /dev/null 2>&1; then
        echo "WARN: NTP not yet synchronized"
        # Don't fail health check, just warn
    fi
fi

echo "OK: All services healthy"
exit 0
