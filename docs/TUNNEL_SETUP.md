# Cloudflare Tunnel Setup Guide

Complete guide for exposing the MCP UTC Time Server via Cloudflare Tunnel for ChatGPT and other remote integrations.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Detailed Setup](#detailed-setup)
5. [ChatGPT Integration](#chatgpt-integration)
6. [Security Best Practices](#security-best-practices)
7. [Troubleshooting](#troubleshooting)
8. [Advanced Configuration](#advanced-configuration)

---

## Overview

### What is Cloudflare Tunnel?

Cloudflare Tunnel (formerly Argo Tunnel) creates a secure, outbound-only connection from your local server to Cloudflare's edge, providing:

- **Public HTTPS access** without exposing ports or configuring firewalls
- **Free tier available** with automatic DDoS protection
- **No domain required** using `.trycloudflare.com` subdomains
- **Automatic SSL/TLS** certificates and encryption

### Architecture

```
Local Machine                 Cloudflare Edge              Internet
┌─────────────────┐          ┌──────────────┐          ┌────────────┐
│ MCP Time Server │          │              │          │            │
│   (STDIO)       │          │  Cloudflare  │          │  ChatGPT   │
│       ↓         │          │    Tunnel    │◄────────►│            │
│   MCPO Wrapper  │──────────►│              │          │  Claude    │
│  (HTTP :8000)   │          │   Gateway    │          │            │
│                 │          │              │          │  Custom    │
└─────────────────┘          └──────────────┘          │  Clients   │
 localhost:8000              https://xxx.try            └────────────┘
                             cloudflare.com
```

---

## Prerequisites

### Required Software

1. **Rust & Cargo** (already installed if you can build the project)
   ```bash
   cargo --version  # Should show 1.75+
   ```

2. **MCPO** (Model Context Protocol over HTTP)
   ```bash
   # Install via pip
   pip install mcpo
   
   # Or via pipx (recommended)
   pipx install mcpo
   
   # Verify installation
   mcpo --version
   # or
   uvx mcpo --help
   ```

3. **cloudflared** (Cloudflare Tunnel client)
   ```bash
   # macOS
   brew install cloudflare/cloudflare/cloudflared
   
   # Linux
   curl -L --output cloudflared https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64
   sudo mv cloudflared /usr/local/bin/
   sudo chmod +x /usr/local/bin/cloudflared
   
   # Windows
   # Download from: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/
   
   # Verify installation
   cloudflared --version
   ```

### Optional Tools

- **jq** - JSON parsing for tests
  ```bash
  brew install jq  # macOS
  sudo apt install jq  # Linux
  ```

- **curl** - HTTP testing (usually pre-installed)

---

## Quick Start

### 1. Generate API Key

```bash
export MCPO_API_KEY="$(openssl rand -base64 32)"
echo "Your API Key: $MCPO_API_KEY"
```

**⚠️ SAVE THIS KEY!** You'll need it for ChatGPT configuration.

### 2. Start the Tunnel

```bash
cd /path/to/mcp-utc-time-server
./scripts/start_tunnel.sh
```

This script will:
- Build the release binary (if needed)
- Start MCPO HTTP wrapper on port 8000
- Launch Cloudflare tunnel
- Display your public HTTPS URL
- Show ChatGPT integration instructions

### 3. Test the Tunnel

In a new terminal:

```bash
export MCPO_API_KEY="your-key-from-step-1"
./scripts/test_tunnel.sh
```

All tests should pass (✓).

### 4. Configure ChatGPT

Follow the on-screen instructions or see [ChatGPT Integration](#chatgpt-integration) below.

---

## Detailed Setup

### Manual Setup (Step-by-Step)

If you prefer manual control over the automated script:

#### Step 1: Build the Project

```bash
cd /path/to/mcp-utc-time-server
cargo build --release
```

Binary will be at: `target/release/mcp-utc-time-server`

#### Step 2: Start MCPO Server

```bash
# Set API key
export MCPO_API_KEY="$(openssl rand -base64 32)"
echo "API Key: $MCPO_API_KEY"  # Save this!

# Start MCPO wrapper
uvx mcpo --port 8000 --api-key "$MCPO_API_KEY" -- \
  ./target/release/mcp-utc-time-server
```

Keep this terminal running.

#### Step 3: Test Local Endpoint

In a new terminal:

```bash
# Health check (no auth)
curl http://localhost:8000/health

# Get OpenAPI schema (no auth)
curl http://localhost:8000/openapi.json | jq .

# Get current time (with auth)
curl -X POST http://localhost:8000/time/get \
  -H "Authorization: Bearer $MCPO_API_KEY" \
  -H "Content-Type: application/json" | jq .
```

#### Step 4: Start Cloudflare Tunnel

In another terminal:

```bash
cloudflared tunnel --url http://localhost:8000
```

Wait for output like:
```
INF +--------------------------------------------------------------------------------------------+
INF |  Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):  |
INF |  https://random-subdomain-1234.trycloudflare.com                                          |
INF +--------------------------------------------------------------------------------------------+
```

**Save this URL!** This is your public endpoint.

#### Step 5: Test Tunnel Endpoint

```bash
TUNNEL_URL="https://your-subdomain.trycloudflare.com"

# Health check
curl "$TUNNEL_URL/health"

# OpenAPI schema
curl "$TUNNEL_URL/openapi.json" | jq '.info'

# Get time (with auth)
curl -X POST "$TUNNEL_URL/time/get" \
  -H "Authorization: Bearer $MCPO_API_KEY" \
  -H "Content-Type: application/json" | jq '.result.iso8601'
```

---

## ChatGPT Integration

### Create Custom GPT

1. **Navigate to GPT Builder**
   - Go to: https://chat.openai.com/gpts/editor
   - Click "Create a GPT"

2. **Configure Basic Info**
   - Name: `UTC Time Server`
   - Description: `Provides precise UTC time with nanosecond precision, timezone conversions, and custom formatting`

3. **Add Actions**
   - Click "Configure" tab
   - Scroll to "Actions" section
   - Click "Add Action"

4. **Set Authentication**
   - Authentication Type: **API Key**
   - Auth Type: **Bearer**
   - API Key: Paste your `MCPO_API_KEY` from earlier
   - Click "Save"

5. **Import OpenAPI Schema**
   - Method 1: Import from URL
     - Click "Import from URL"
     - Enter: `https://your-tunnel-url.trycloudflare.com/openapi.json`
     - Click "Import"
   
   - Method 2: Copy/paste schema
     ```bash
     curl https://your-tunnel-url.trycloudflare.com/openapi.json > schema.json
     ```
     - Copy contents and paste into the schema editor

6. **Edit Schema (CRITICAL)**
   
   After importing, you MUST add the `servers` array. Find the line:
   ```json
   "openapi": "3.0.0",
   ```
   
   Immediately after it, add:
   ```json
   "servers": [
     {
       "url": "https://your-actual-tunnel-url.trycloudflare.com"
     }
   ],
   ```
   
   **Important:**
   - Replace `your-actual-tunnel-url` with your real tunnel URL
   - NO trailing slash
   - Use HTTPS (not HTTP)

7. **Test the GPT**
   
   Try these prompts:
   ```
   What's the current UTC time?
   
   Give me the time in Tokyo right now
   
   What's the Unix timestamp with nanosecond precision?
   
   Format the current time as YYYY-MM-DD HH:MM:SS
   
   List timezones starting with "America/"
   ```

8. **Publish (Optional)**
   - Click "Save"
   - Choose visibility: Only Me, Anyone with Link, or Public
   - Click "Confirm"

### Example GPT Configuration

**Instructions for GPT:**
```
You are a precise time server assistant. You provide:
- Current UTC time with nanosecond precision
- Timezone conversions for 595+ IANA timezones
- Custom time formatting using strftime
- Unix timestamps
- Time component extraction

Always format times clearly and explain timezone offsets when relevant.
Use the time server tools to get real-time data.
```

**Conversation Starters:**
```
What's the current UTC time?
Convert current time to Tokyo timezone
Get Unix timestamp in nanoseconds
Format time as ISO 8601
```

---

## Security Best Practices

### 1. API Key Management

**Generate Strong Keys:**
```bash
# 32-byte base64 key (recommended)
openssl rand -base64 32

# Or use UUID
uuidgen
```

**Store Securely:**
```bash
# Use environment file (DO NOT commit to git!)
echo "export MCPO_API_KEY='your-key'" >> ~/.mcpo_env
echo ".mcpo_env" >> .gitignore

# Load when needed
source ~/.mcpo_env
```

**Rotate Regularly:**
```bash
# Generate new key
NEW_KEY=$(openssl rand -base64 32)

# Update environment
export MCPO_API_KEY="$NEW_KEY"

# Restart servers with new key
./scripts/stop_tunnel.sh
./scripts/start_tunnel.sh

# Update ChatGPT GPT with new key
```

### 2. Access Control

**Monitor Access:**
```bash
# Watch MCPO logs for unauthorized attempts
tail -f mcpo.log | grep -E "(401|403|ERROR)"
```

**Rate Limiting (Optional):**
```bash
# Add rate limiting with nginx proxy
# See Advanced Configuration section
```

### 3. Network Security

**Firewall Rules:**
```bash
# No incoming ports needed! Tunnel is outbound-only
# But if running MCPO separately:
sudo ufw allow 8000/tcp  # Only if needed
```

**HTTPS Only:**
- Cloudflare Tunnel automatically uses HTTPS
- Never expose raw HTTP to the internet
- MCPO runs on localhost only

### 4. Monitoring

**Check Running Processes:**
```bash
# List tunnel and MCPO processes
ps aux | grep -E "(cloudflared|mcpo)"

# Check listening ports
lsof -i :8000
```

**Review Logs:**
```bash
# MCPO logs
cat mcpo.log | grep -E "(ERROR|WARN)"

# Tunnel logs
cat tunnel.log | tail -n 50
```

---

## Troubleshooting

### Common Issues

#### 1. Tunnel URL Not Appearing

**Symptoms:**
```
Waiting for tunnel URL...
.......................
ERROR: Failed to get tunnel URL
```

**Solutions:**
```bash
# Check if cloudflared is running
ps aux | grep cloudflared

# Check tunnel log
cat tunnel.log

# Test cloudflared directly
cloudflared tunnel --url http://localhost:8000

# Update cloudflared
brew upgrade cloudflared  # macOS
```

#### 2. MCPO Server Won't Start

**Symptoms:**
```
ERROR: MCPO server failed to start
```

**Solutions:**
```bash
# Check if port is in use
lsof -i :8000

# Kill existing process
pkill -f "mcpo"

# Check MCPO log
cat mcpo.log

# Try different port
export MCPO_PORT=8001
./scripts/start_tunnel.sh
```

#### 3. 401 Unauthorized from Tunnel

**Symptoms:**
```
curl: (22) The requested URL returned error: 401
```

**Solutions:**
```bash
# Verify API key matches
echo "Server: Check mcpo.log for configured key"
echo "Client: Check your MCPO_API_KEY environment variable"

# Test with correct key
curl -X POST https://tunnel-url/time/get \
  -H "Authorization: Bearer $MCPO_API_KEY" \
  -H "Content-Type: application/json"

# Regenerate if lost
export MCPO_API_KEY="$(openssl rand -base64 32)"
# Restart servers with new key
```

#### 4. ChatGPT Can't Access Server

**Symptoms:**
- GPT returns "I couldn't reach the time server"
- Actions show errors in ChatGPT interface

**Solutions:**

1. **Verify tunnel is accessible:**
   ```bash
   curl https://your-tunnel-url.trycloudflare.com/health
   ```

2. **Check servers array in OpenAPI JSON:**
   - Go to GPT → Configure → Actions
   - Verify `servers` array exists with correct URL
   - NO trailing slash

3. **Test API key:**
   ```bash
   curl -X POST https://your-tunnel-url/time/get \
     -H "Authorization: Bearer YOUR_GPT_API_KEY" \
     -H "Content-Type: application/json"
   ```

4. **Re-import schema:**
   - Delete existing action
   - Import fresh from `/openapi.json`
   - Add servers array again

#### 5. Tunnel Disconnects

**Symptoms:**
```
Connection to Cloudflare lost
Tunnel disconnected
```

**Solutions:**
```bash
# Check network connectivity
ping 1.1.1.1

# Restart tunnel
./scripts/stop_tunnel.sh
./scripts/start_tunnel.sh

# Use persistent tunnel (see Advanced Configuration)
cloudflared tunnel login
cloudflared tunnel create mcp-time-server
```

### Debug Mode

**Enable verbose logging:**

```bash
# Start with debug output
export RUST_LOG=debug
export MCPO_DEBUG=1

./scripts/start_tunnel.sh

# Watch logs in real-time
tail -f mcpo.log tunnel.log
```

**Test each component separately:**

```bash
# 1. Test binary
./target/release/mcp-utc-time-server
# Enter: {"jsonrpc":"2.0","method":"time/get","params":{},"id":1}

# 2. Test MCPO (without tunnel)
uvx mcpo --port 8000 --api-key test-key -- \
  ./target/release/mcp-utc-time-server &
curl http://localhost:8000/health

# 3. Test tunnel (separate from MCPO)
# Start any HTTP server on 8000, then:
cloudflared tunnel --url http://localhost:8000
```

---

## Advanced Configuration

### Persistent Named Tunnels

For production use, create a persistent tunnel:

```bash
# 1. Authenticate with Cloudflare
cloudflared tunnel login

# 2. Create named tunnel
cloudflared tunnel create mcp-time-server

# 3. Create config file
mkdir -p ~/.cloudflared
cat > ~/.cloudflared/config.yml <<EOF
tunnel: <TUNNEL-ID-FROM-STEP-2>
credentials-file: /Users/yourname/.cloudflared/<TUNNEL-ID>.json

ingress:
  - hostname: time.your-domain.com
    service: http://localhost:8000
  - service: http_status:404
EOF

# 4. Create DNS record
cloudflared tunnel route dns mcp-time-server time.your-domain.com

# 5. Run tunnel
cloudflared tunnel run mcp-time-server
```

### Systemd Service (Linux)

Create `/etc/systemd/system/mcp-tunnel.service`:

```ini
[Unit]
Description=MCP UTC Time Server Tunnel
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/opt/mcp-utc-time-server
Environment="MCPO_API_KEY=your-secure-key"
Environment="RUST_LOG=info"

ExecStart=/usr/local/bin/start_tunnel.sh
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable mcp-tunnel
sudo systemctl start mcp-tunnel
sudo systemctl status mcp-tunnel
```

### Docker Deployment

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  mcp-server:
    build: .
    environment:
      - RUST_LOG=info
    stdin_open: true
    tty: true
    networks:
      - internal

  mcpo:
    image: ghcr.io/serenaai/mcpo:latest
    command: ["--port", "8000", "--api-key", "${MCPO_API_KEY}", "--", "mcp-server"]
    environment:
      - MCPO_API_KEY=${MCPO_API_KEY}
    ports:
      - "8000:8000"
    depends_on:
      - mcp-server
    networks:
      - internal

  tunnel:
    image: cloudflare/cloudflared:latest
    command: ["tunnel", "--url", "http://mcpo:8000"]
    depends_on:
      - mcpo
    networks:
      - internal

networks:
  internal:
    driver: bridge
```

Run with:
```bash
export MCPO_API_KEY="$(openssl rand -base64 32)"
docker-compose up -d
docker-compose logs -f tunnel  # Get tunnel URL
```

### Rate Limiting with Nginx

If you need rate limiting:

```nginx
http {
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    
    server {
        listen 9000;
        
        location / {
            limit_req zone=api_limit burst=20 nodelay;
            proxy_pass http://localhost:8000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

Then tunnel to nginx:
```bash
cloudflared tunnel --url http://localhost:9000
```

### Custom Domain

Instead of `.trycloudflare.com`:

1. **Add site to Cloudflare:**
   - Go to Cloudflare dashboard
   - Add your domain
   - Update nameservers

2. **Create tunnel:**
   ```bash
   cloudflared tunnel create mcp-time
   ```

3. **Configure DNS:**
   ```bash
   cloudflared tunnel route dns mcp-time api.yourdomain.com
   ```

4. **Update config:**
   ```yaml
   # ~/.cloudflared/config.yml
   tunnel: <tunnel-id>
   credentials-file: /path/to/credentials.json
   
   ingress:
     - hostname: api.yourdomain.com
       service: http://localhost:8000
     - service: http_status:404
   ```

5. **Run:**
   ```bash
   cloudflared tunnel run mcp-time
   ```

---

## Monitoring & Maintenance

### Health Checks

```bash
# Automated health check script
#!/bin/bash
TUNNEL_URL="https://your-tunnel.trycloudflare.com"

while true; do
    if curl -s -f "$TUNNEL_URL/health" > /dev/null; then
        echo "$(date): ✓ Healthy"
    else
        echo "$(date): ✗ Unhealthy - Restarting..."
        ./scripts/stop_tunnel.sh
        ./scripts/start_tunnel.sh
    fi
    sleep 60
done
```

### Log Rotation

```bash
# Add to crontab
0 0 * * * mv ~/mcp-utc-time-server/mcpo.log ~/mcp-utc-time-server/mcpo.log.$(date +\%Y\%m\%d)
0 0 * * * mv ~/mcp-utc-time-server/tunnel.log ~/mcp-utc-time-server/tunnel.log.$(date +\%Y\%m\%d)
0 0 * * 0 find ~/mcp-utc-time-server -name "*.log.*" -mtime +7 -delete
```

### Metrics Collection

```bash
# Simple metrics script
#!/bin/bash
LOG_FILE="mcpo.log"

echo "=== MCP Server Metrics ==="
echo "Total requests: $(grep -c "POST /time" $LOG_FILE)"
echo "Successful: $(grep -c "200 OK" $LOG_FILE)"
echo "Unauthorized: $(grep -c "401" $LOG_FILE)"
echo "Errors: $(grep -c "ERROR" $LOG_FILE)"
echo "Unique IPs: $(grep "POST /time" $LOG_FILE | awk '{print $1}' | sort -u | wc -l)"
```

---

## Summary

You now have a production-ready Cloudflare Tunnel setup for your MCP UTC Time Server!

**Quick Commands Reference:**

```bash
# Start everything
./scripts/start_tunnel.sh

# Test connectivity
./scripts/test_tunnel.sh

# Stop everything
./scripts/stop_tunnel.sh

# Manual testing
curl https://tunnel-url/health
curl https://tunnel-url/openapi.json
curl -X POST https://tunnel-url/time/get \
  -H "Authorization: Bearer $MCPO_API_KEY"
```

**Key Files:**
- `scripts/start_tunnel.sh` - All-in-one startup
- `scripts/test_tunnel.sh` - Comprehensive tests
- `scripts/stop_tunnel.sh` - Clean shutdown
- `.tunnel_url` - Saved tunnel URL
- `mcpo.log` - MCPO server logs
- `tunnel.log` - Cloudflare tunnel logs

**Support:**
- GitHub Issues: Report problems
- Documentation: `/docs` directory
- Examples: `/examples` and `/scripts`

Happy tunneling! 🚀
