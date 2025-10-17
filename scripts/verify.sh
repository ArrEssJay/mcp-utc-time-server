#!/bin/bash
# Quick verification test for MCP UTC Time Server

echo "=== MCP UTC Time Server - Quick Verification ==="
echo ""

# Test 1: Build
echo "1. Testing build..."
if cargo build --release > /dev/null 2>&1; then
    echo "   ✅ Build successful"
else
    echo "   ❌ Build failed"
    exit 1
fi

# Test 2: Run tests
echo "2. Running unit tests..."
if cargo test --quiet > /dev/null 2>&1; then
    echo "   ✅ All tests pass"
else
    echo "   ❌ Tests failed"
    exit 1
fi

# Test 3: Start server and check health
echo "3. Testing server startup..."
timeout 5s cargo run --release > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
    echo "   ✅ Health endpoint responding"
else
    echo "   ❌ Health endpoint not responding"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# Test 4: Check metrics
echo "4. Testing metrics endpoint..."
if curl -sf http://localhost:3000/metrics | grep -q "mcp_time_seconds"; then
    echo "   ✅ Metrics endpoint working"
else
    echo "   ❌ Metrics endpoint failed"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# Cleanup
kill $SERVER_PID 2>/dev/null
sleep 1

echo ""
echo "=== All Checks Passed! ==="
echo ""
echo "Server is ready for:"
echo "  • Cloud deployment (docker-compose up)"
echo "  • Edge deployment (docker-compose -f docker-compose.rpi.yml up)"
echo "  • MCP client usage (Claude Desktop, etc.)"
echo ""
