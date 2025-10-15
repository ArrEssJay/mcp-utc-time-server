# Scripts Reference

Quick reference for all helper scripts in the MCP UTC Time Server project.

## Tunnel Management (Cloudflare)

### `start_tunnel.sh` - Automated Tunnel Setup

**Purpose:** Starts MCPO HTTP wrapper and Cloudflare tunnel with full automation.

**Usage:**
```bash
export MCPO_API_KEY="$(openssl rand -base64 32)"
./scripts/start_tunnel.sh
```

**What it does:**
- Checks dependencies (mcpo, cloudflared)
- Builds release binary if needed
- Starts MCPO on port 8000
- Launches Cloudflare tunnel
- Displays public HTTPS URL
- Shows ChatGPT setup instructions
- Streams live logs

**Output files:**
- `.tunnel_url` - Saved tunnel URL
- `mcpo.log` - MCPO server logs
- `tunnel.log` - Cloudflare tunnel logs

**Environment variables:**
- `MCPO_API_KEY` - Required, your secret API key
- `MCPO_PORT` - Optional, default: 8000

**Keyboard shortcuts:**
- `Ctrl+C` - Gracefully stop both servers

---

### `test_tunnel.sh` - Tunnel Connectivity Tests

**Purpose:** Comprehensive testing of tunnel endpoints and API functionality.

**Usage:**
```bash
export MCPO_API_KEY="your-key"
./scripts/test_tunnel.sh
```

**Prerequisites:**
- Tunnel must be running (`start_tunnel.sh`)
- `.tunnel_url` file must exist
- `MCPO_API_KEY` must be set

**Tests performed:**
1. Health check (no auth)
2. OpenAPI schema retrieval
3. Unauthorized access (expects 401)
4. Get current time
5. Get Unix timestamp
6. Get nanoseconds
7. Custom format (strftime)
8. Timezone conversion
9. List timezones
10. Convert timezone

**Exit codes:**
- `0` - All tests passed
- `1` - One or more tests failed

---

### `stop_tunnel.sh` - Stop All Tunnel Services

**Purpose:** Gracefully stops MCPO and cloudflared processes.

**Usage:**
```bash
./scripts/stop_tunnel.sh
```

**What it does:**
- Finds and kills cloudflared tunnel processes
- Finds and kills MCPO server processes
- Removes `.tunnel_url` file
- Optionally removes log files

**Interactive prompts:**
- Asks before removing log files

---

## MCPO (HTTP Wrapper)

### `start_mcpo.sh` - Start MCPO Server

**Purpose:** Starts MCPO HTTP wrapper for local API access (without tunnel).

**Usage:**
```bash
export MCPO_API_KEY="your-secret-key"
./scripts/start_mcpo.sh
```

**Configuration:**
- `MCPO_PORT` - Default: 8000
- `MCPO_API_KEY` - Required

**Use cases:**
- Local development
- Internal network API
- Testing before exposing via tunnel
- Use with separate tunnel setup

**Access:**
```bash
curl http://localhost:8000/health
curl http://localhost:8000/openapi.json
```

---

### `test_mcpo.sh` - Test MCPO HTTP API

**Purpose:** Tests MCPO server on localhost.

**Usage:**
```bash
export MCPO_API_KEY="your-key"
./scripts/test_mcpo.sh
```

**Prerequisites:**
- MCPO server running on localhost:8000
- `MCPO_API_KEY` environment variable set

**Tests:**
- All time/* endpoints
- Authentication
- JSON response validation
- Error handling

---

## VSCode Integration

### `test_vscode.sh` - Test VSCode MCP Integration

**Purpose:** Tests the MCP server via STDIO (as VSCode uses it).

**Usage:**
```bash
./scripts/test_vscode.sh
```

**What it does:**
- Spawns MCP server as subprocess
- Sends JSON-RPC requests via STDIN
- Captures and validates STDOUT responses
- Tests all 7 MCP methods:
  1. `initialize`
  2. `time/get`
  3. `time/get_unix`
  4. `time/get_nanos`
  5. `time/get_with_format`
  6. `time/get_with_timezone`
  7. `time/list_timezones`

**No prerequisites:**
- Automatically builds binary if needed
- No external dependencies required

---

## Quick Command Reference

```bash
# === Tunnel Setup (ChatGPT Integration) ===

# 1. Generate API key
export MCPO_API_KEY="$(openssl rand -base64 32)"

# 2. Start tunnel
./scripts/start_tunnel.sh

# 3. Test (in another terminal)
./scripts/test_tunnel.sh

# 4. Stop
./scripts/stop_tunnel.sh


# === Local MCPO (HTTP API) ===

# 1. Start server
export MCPO_API_KEY="my-secret-key"
./scripts/start_mcpo.sh

# 2. Test
./scripts/test_mcpo.sh


# === VSCode Integration ===

# Just test it
./scripts/test_vscode.sh
```

---

## Troubleshooting

### "MCPO_API_KEY not set"

```bash
# Generate new key
export MCPO_API_KEY="$(openssl rand -base64 32)"

# Or set custom key
export MCPO_API_KEY="my-secret-key-here"

# Add to shell profile for persistence
echo 'export MCPO_API_KEY="your-key"' >> ~/.zshrc
```

### "Port 8000 already in use"

```bash
# Find what's using it
lsof -i :8000

# Kill the process
kill $(lsof -t -i :8000)

# Or use different port
export MCPO_PORT=8001
./scripts/start_tunnel.sh
```

### "cloudflared not found"

```bash
# macOS
brew install cloudflare/cloudflare/cloudflared

# Linux
curl -L --output cloudflared \
  https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64
sudo mv cloudflared /usr/local/bin/
sudo chmod +x /usr/local/bin/cloudflared
```

### "mcpo not found"

```bash
# Install with pip
pip install mcpo

# Or with pipx
pipx install mcpo

# Verify
mcpo --version
# or
uvx mcpo --help
```

### Tunnel URL not appearing

```bash
# Check tunnel log
cat tunnel.log

# Test cloudflared directly
cloudflared tunnel --url http://localhost:8000

# Update cloudflared
brew upgrade cloudflared  # macOS
```

### Tests failing

```bash
# Enable debug mode
export RUST_LOG=debug
export MCPO_DEBUG=1

# Check logs
cat mcpo.log
cat tunnel.log

# Test binary directly
./target/release/mcp-utc-time-server
# Then type: {"jsonrpc":"2.0","method":"time/get","params":{},"id":1}
```

---

## Log Files

All scripts write to these locations:

| File | Purpose | Location |
|------|---------|----------|
| `mcpo.log` | MCPO server output | Project root |
| `tunnel.log` | Cloudflare tunnel output | Project root |
| `.tunnel_url` | Current tunnel URL | Project root (auto-generated) |

**Viewing logs:**
```bash
# Watch MCPO logs
tail -f mcpo.log

# Watch tunnel logs
tail -f tunnel.log

# View recent errors
grep -E "(ERROR|WARN)" mcpo.log

# Clean up old logs
rm -f *.log .tunnel_url
```

---

## Environment Variables

### Required

| Variable | Used By | Purpose |
|----------|---------|---------|
| `MCPO_API_KEY` | All MCPO scripts | Authentication key for HTTP API |

### Optional

| Variable | Default | Purpose |
|----------|---------|---------|
| `MCPO_PORT` | 8000 | HTTP server port |
| `RUST_LOG` | info | Log level (trace/debug/info/warn/error) |
| `MCPO_DEBUG` | (unset) | Enable MCPO debug output |
| `TZ` | UTC | System timezone |

---

## Script Permissions

All scripts should be executable:

```bash
chmod +x scripts/*.sh

# Verify
ls -l scripts/
```

If you get "Permission denied":
```bash
bash ./scripts/start_tunnel.sh
```

---

## Integration Guide

For complete integration documentation:

- **Tunnel Setup:** [docs/TUNNEL_SETUP.md](../docs/TUNNEL_SETUP.md)
- **All Integrations:** [docs/INTEGRATION.md](../docs/INTEGRATION.md)
- **Quick Start:** [QUICKSTART.md](../QUICKSTART.md)
- **Main README:** [README.md](../README.md)

---

## Support

For issues or questions:

1. Check logs: `cat mcpo.log tunnel.log`
2. Review documentation in `/docs`
3. Run tests with debug: `RUST_LOG=debug ./scripts/test_*.sh`
4. Open GitHub issue with logs and error messages
