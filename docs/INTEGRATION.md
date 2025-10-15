# VSCode & ChatGPT Integration Guide for MCP UTC Time Server

## Overview

This guide covers three integration scenarios:
1. **Direct VSCode Integration** - Local MCP server for VSCode AI features
2. **MCPO HTTP Wrapper** - Expose MCP server as HTTP API for ChatGPT/Claude
3. **Cloudflare Tunnel** - Secure public access for remote AI agents

---

## 1. VSCode Integration (Local MCP)

### Prerequisites
- VSCode with MCP support
- Rust toolchain installed
- Project built with `cargo build --release`

### Configuration

#### Option A: Direct Cargo Run (Development)

Add to your VSCode `settings.json`:

```json
{
    "mcp.servers": {
        "utc-time": {
            "command": "cargo",
            "args": ["run", "--release"],
            "cwd": "/absolute/path/to/mcp-utc-time-server",
            "env": {
                "RUST_LOG": "info",
                "TZ": "UTC"
            }
        }
    }
}
```

#### Option B: Compiled Binary (Production)

```json
{
    "mcp.servers": {
        "utc-time": {
            "command": "/absolute/path/to/target/release/mcp-utc-time-server",
            "args": [],
            "env": {
                "RUST_LOG": "info",
                "TZ": "UTC"
            }
        }
    }
}
```

### Testing VSCode Integration

1. **Reload VSCode**: Run `Developer: Reload Window` (Cmd+Shift+P)
2. **Verify Server**: Check output panel for "MCP UTC Time Server started"
3. **Test from AI Chat**:
   ```
   Get the current UTC time with nanosecond precision
   ```

### Available Commands in VSCode

Once integrated, AI assistants can use these tools:

- `time/get` - Complete time information
- `time/get_unix` - Unix timestamp with nanoseconds
- `time/get_with_format` - Custom strftime formatting
- `time/get_with_timezone` - Time in any timezone
- `time/list_timezones` - List all IANA timezones
- `time/convert` - Convert between timezones

---

## 2. MCPO Integration (HTTP API Wrapper)

### What is MCPO?

MCPO (Model Context Protocol over HTTP) wraps STDIO-based MCP servers into HTTP APIs, enabling integration with ChatGPT Custom GPTs, Claude via API, and other HTTP-based AI systems.

### Installation

```bash
# Install mcpo using uvx
pip install mcpo
# or
uvx mcpo --help
```

### Starting the Server with MCPO

```bash
# Navigate to project directory
cd /path/to/mcp-utc-time-server

# Build release binary first
cargo build --release

# Start via MCPO with authentication
uvx mcpo --port 8000 --api-key YOUR_SECRET_KEY -- \
  /absolute/path/to/target/release/mcp-utc-time-server
```

**Important Parameters:**
- `--port 8000` - HTTP server port
- `--api-key YOUR_SECRET_KEY` - Required for security (use a strong random key)
- Everything after `--` is the command to run your MCP server

### Testing MCPO Server

```bash
# Check server health
curl http://localhost:8000/health

# Get OpenAPI schema
curl http://localhost:8000/openapi.json

# Call time/get method
curl -X POST http://localhost:8000/time/get \
  -H "Authorization: Bearer YOUR_SECRET_KEY" \
  -H "Content-Type: application/json"
```

### Environment Variables

Create a `.env` file for configuration:

```bash
# .env
MCPO_PORT=8000
MCPO_API_KEY=your-secret-key-here
RUST_LOG=info
TZ=UTC
```

Then start with:

```bash
source .env
uvx mcpo --port $MCPO_PORT --api-key $MCPO_API_KEY -- \
  cargo run --release
```

---

## 3. ChatGPT Integration via Cloudflare Tunnel

### Prerequisites

```bash
# Install cloudflared
brew install cloudflare/cloudflare/cloudflared
# or download from: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/
```

### Step-by-Step Setup

#### Step 1: Start MCPO Server

```bash
cd /path/to/mcp-utc-time-server
cargo build --release

# Start with a secure API key
uvx mcpo --port 8000 --api-key "$(openssl rand -base64 32)" -- \
  ./target/release/mcp-utc-time-server
```

**Save your API key!** You'll need it for ChatGPT authentication.

#### Step 2: Create Cloudflare Tunnel

```bash
# In a new terminal
cloudflared tunnel --url http://localhost:8000
```

You'll get a URL like:
```
https://random-subdomain.trycloudflare.com
```

**Keep this terminal running!**

#### Step 3: Configure ChatGPT Custom GPT

1. Go to [ChatGPT → Create GPT](https://chat.openai.com/gpts/editor)
2. Click **"Configure"** → **"Actions"** → **"Add Action"**
3. Set **Authentication**:
   - Type: **API Key**
   - Auth Type: **Bearer**
   - API Key: Your `MCPO_API_KEY` from Step 1

4. Import OpenAPI Schema:
   - Click **"Import from URL"**
   - Enter: `https://your-tunnel-url.trycloudflare.com/openapi.json`

5. **Critical**: Edit the imported JSON schema
   - Add at the top (after `"openapi": "3.0.0"`):
   ```json
   "servers": [
     {
       "url": "https://your-tunnel-url.trycloudflare.com"
     }
   ],
   ```
   - **Do not include a trailing slash!**

6. **Save and Test**:
   ```
   What's the current UTC time in nanoseconds?
   Get the time in Tokyo timezone
   Format the current time as YYYY-MM-DD HH:MM:SS
   ```

### Security Considerations

⚠️ **IMPORTANT SECURITY WARNINGS**

1. **API Key Protection**
   - Use a strong, randomly generated API key
   - Never commit API keys to version control
   - Rotate keys regularly
   - Use environment variables

2. **Network Exposure**
   - The tunnel exposes your server to the internet
   - Only run when needed
   - Monitor access logs
   - Consider IP allowlisting

3. **Rate Limiting**
   ```bash
   # Add rate limiting with nginx proxy (optional)
   uvx mcpo --port 8000 --api-key $API_KEY \
     --rate-limit 100 \
     -- cargo run --release
   ```

4. **Read-Only Mode**
   - This time server is read-only by design
   - No file system access
   - No command execution
   - Safe for public exposure with proper authentication

---

## 4. Testing All Integration Methods

### Test Script for MCPO

Create `test_mcpo.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

API_KEY="${MCPO_API_KEY:-test-key}"
BASE_URL="${MCPO_URL:-http://localhost:8000}"

echo "Testing MCPO Integration..."
echo

echo "1. Health Check:"
curl -s "$BASE_URL/health" | jq '.'
echo

echo "2. Get UTC Time:"
curl -s -X POST "$BASE_URL/time/get" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" | jq '.result | {unix: .unix.seconds, iso8601, timezone}'
echo

echo "3. Get Unix Timestamp:"
curl -s -X POST "$BASE_URL/time/get_unix" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" | jq '.result'
echo

echo "4. Custom Format:"
curl -s -X POST "$BASE_URL/time/get_with_format" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"format": "%Y-%m-%d %H:%M:%S"}' | jq '.result.formatted'
echo

echo "All tests completed!"
```

```bash
chmod +x test_mcpo.sh
export MCPO_API_KEY="your-key-here"
./test_mcpo.sh
```

### VSCode Extension Testing

Create `.vscode/extensions.json`:

```json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "tamasfe.even-better-toml"
    ]
}
```

### Integration Test Checklist

- [ ] VSCode direct integration works
- [ ] MCPO HTTP wrapper responds correctly
- [ ] OpenAPI schema is valid
- [ ] API key authentication works
- [ ] Cloudflare tunnel establishes successfully
- [ ] ChatGPT can call all methods
- [ ] Error handling works properly
- [ ] Logging captures requests

---

## 5. Production Deployment

### Systemd Service (Linux)

Create `/etc/systemd/system/mcp-time-server.service`:

```ini
[Unit]
Description=MCP UTC Time Server
After=network.target

[Service]
Type=simple
User=mcp-server
WorkingDirectory=/opt/mcp-utc-time-server
Environment="RUST_LOG=info"
Environment="TZ=UTC"
ExecStart=/opt/mcp-utc-time-server/target/release/mcp-utc-time-server
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable mcp-time-server
sudo systemctl start mcp-time-server
sudo systemctl status mcp-time-server
```

### Docker Container

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mcp-utc-time-server /usr/local/bin/
ENV RUST_LOG=info
ENV TZ=UTC
CMD ["mcp-utc-time-server"]
```

Build and run:
```bash
docker build -t mcp-utc-time-server .
docker run -i mcp-utc-time-server
```

### MCPO with Docker

```bash
# Run MCP server in Docker
docker run -d --name mcp-time mcp-utc-time-server

# Wrap with MCPO
uvx mcpo --port 8000 --api-key $API_KEY -- \
  docker attach mcp-time
```

---

## 6. Monitoring & Debugging

### Enable Debug Logging

```bash
# VSCode settings.json
{
    "mcp.servers": {
        "utc-time": {
            "env": {
                "RUST_LOG": "mcp_utc_time_server=trace,debug"
            }
        }
    }
}
```

### Log Files

```bash
# Capture logs to file
cargo run --release 2> server.log

# Watch logs in real-time
tail -f server.log | grep -E "(ERROR|WARN|INFO)"
```

### Request Tracing

The server outputs structured logs:
```
[INFO] MCP UTC Time Server started
[DEBUG] Received request: {"jsonrpc":"2.0","method":"time/get",...}
[DEBUG] Sending response: {"jsonrpc":"2.0","result":{...}}
```

### Performance Metrics

```bash
# Monitor with hyperfine
hyperfine --warmup 10 \
  'echo "{\"jsonrpc\":\"2.0\",\"method\":\"time/get\",\"params\":{},\"id\":1}" | cargo run --release'
```

---

## 7. Troubleshooting

### VSCode Issues

**Server Not Starting:**
```bash
# Check binary exists and is executable
ls -l target/release/mcp-utc-time-server
file target/release/mcp-utc-time-server

# Test manually
./target/release/mcp-utc-time-server
# Type: {"jsonrpc":"2.0","method":"time/get","params":{},"id":1}
# Press Enter
```

**No Response from Server:**
- Check VSCode Output panel → MCP Servers
- Verify PATH includes cargo/rustc
- Check RUST_LOG environment variable

### MCPO Issues

**Port Already in Use:**
```bash
# Find process using port 8000
lsof -i :8000
# Kill it or use different port
uvx mcpo --port 8001 ...
```

**API Key Authentication Failing:**
```bash
# Test without authentication
curl -X POST http://localhost:8000/time/get \
  -H "Content-Type: application/json"
# Should return 401 Unauthorized

# Test with correct key
curl -X POST http://localhost:8000/time/get \
  -H "Authorization: Bearer YOUR_KEY" \
  -H "Content-Type: application/json"
```

### Cloudflare Tunnel Issues

**Tunnel Not Connecting:**
```bash
# Check cloudflared is running
ps aux | grep cloudflared

# Verify local server is accessible
curl http://localhost:8000/health

# Check firewall rules
sudo lsof -i :8000
```

**ChatGPT Can't Reach Server:**
1. Verify tunnel URL is accessible: `curl https://your-tunnel.trycloudflare.com/openapi.json`
2. Check API key is correct in ChatGPT settings
3. Ensure `servers` array is in OpenAPI JSON
4. No trailing slashes in URLs

---

## 8. Best Practices

### Development
- Use cargo run for quick iteration
- Enable debug logging during development
- Test with `examples/demo.sh` regularly

### Production
- Always use compiled binary (`--release`)
- Set up proper logging and monitoring
- Use systemd or Docker for process management
- Implement rate limiting for public APIs

### Security
- Never expose without authentication
- Use strong, random API keys (32+ bytes)
- Rotate keys regularly
- Monitor access logs
- Run with least privileges
- Consider running in container

### Performance
- Precompile with `--release` flag
- Use binary distribution, not cargo run
- Monitor memory usage (should stay <50MB)
- Set up health checks

---

## 9. Example Use Cases

### VSCode Copilot Integration

In your VSCode chat:
```
@utc-time Get the current time in RFC 3339 format

@utc-time What timezone is EST and what's the current time there?

@utc-time Convert Unix timestamp 1700000000 to Tokyo time
```

### ChatGPT Custom GPT

Example prompts:
```
What's the exact UTC time right now in nanoseconds?

Give me the current time formatted for an Apache log

What's the time difference between New York and Tokyo right now?

Show me all timezone abbreviations starting with "America/"
```

### API Integration

```python
import requests

API_KEY = "your-secret-key"
BASE_URL = "https://your-tunnel.trycloudflare.com"

def get_utc_time():
    response = requests.post(
        f"{BASE_URL}/time/get",
        headers={"Authorization": f"Bearer {API_KEY}"}
    )
    return response.json()["result"]

# Get Unix timestamp
def get_unix_time():
    response = requests.post(
        f"{BASE_URL}/time/get_unix",
        headers={"Authorization": f"Bearer {API_KEY}"}
    )
    return response.json()["result"]["seconds"]
```

---

## Summary

This MCP UTC Time Server can be integrated in multiple ways:

1. **VSCode** - Direct STDIO integration for local AI assistants
2. **MCPO** - HTTP wrapper for web-based AI services
3. **ChatGPT** - Custom GPT with Cloudflare tunnel

All methods provide the same capabilities:
- Nanosecond precision time
- Multiple format support
- Timezone conversions
- Unix/POSIX compliance

Choose the integration method that best fits your use case!
