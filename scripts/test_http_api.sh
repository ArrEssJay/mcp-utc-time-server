#!/bin/bash
# Test runner for HTTP API tests

set -e

echo "🧪 Running HTTP API Integration Tests"
echo "======================================"
echo ""

# Set test environment
export RUST_LOG=info
export RUST_BACKTRACE=1

# Run the tests
echo "Running tests with cargo test..."
cargo test --test http_api_test -- --nocapture

echo ""
echo "✅ All tests passed!"
