# Quick Start Guide - MCP UTC Time Server

This guide gets you up and running in 5 minutes.

## Installation

```bash
# Clone repository
git clone https://github.com/yourusername/mcp-utc-time-server
cd mcp-utc-time-server

# Build release version
cargo build --release

# Run tests to verify
cargo test --all
```

## Usage Options

### 1. Command Line (Quickest)

```bash
# Interactive mode
./target/release/mcp-utc-time-server

# Type this and press Enter:
{"jsonrpc":"2.0","method":"time/get","params":{},"id":1}
```

### 2. VSCode Integration (Recommended)

**Add to `.vscode/settings.json`:**

```json
{
    "mcp.servers": {
        "utc-time": {
            "command": "/absolute/path/to/mcp-utc-time-server/target/release/mcp-utc-time-server",
            "env": {
                "RUST_LOG": "info"
            }
        }
    }
}
```

**Test it:**

```bash
./scripts/test_vscode.sh
```

All tests should show ✓ (green checkmarks).

**In VSCode:**
- Reload window (Cmd+Shift+P → "Developer: Reload Window")
- Open Copilot chat
- Type: "Get current UTC time"

### 3. ChatGPT Integration (Advanced)

**Install MCPO:**

```bash
pip install mcpo
# or
brew install mcpo
```

**Start server:**

```bash
export MCPO_API_KEY="$(openssl rand -base64 32)"
./scripts/start_mcpo.sh
```

**Expose with Cloudflare:**

```bash
# In new terminal
cloudflared tunnel --url http://localhost:8000
```

**Configure ChatGPT:**
See [docs/INTEGRATION.md](docs/INTEGRATION.md#3-chatgpt-integration-via-cloudflare-tunnel) for detailed steps.

## Available Methods

| Method | Description | Example |
|--------|-------------|---------|
| `time/get` | Complete time info | All formats, components, timezones |
| `time/get_unix` | Unix timestamp | Seconds + nanoseconds |
| `time/get_nanos` | Nanoseconds since epoch | High precision timing |
| `time/get_with_format` | Custom strftime | Format: `%Y-%m-%d %H:%M:%S` |
| `time/get_with_timezone` | Time in timezone | `Asia/Tokyo`, `America/New_York` |
| `time/list_timezones` | All IANA timezones | 595+ timezone identifiers |
| `time/convert` | Convert between TZs | UTC → any timezone |

## Quick Examples

### Get Current Time
```bash
echo '{"jsonrpc":"2.0","method":"time/get","params":{},"id":1}' | \
  ./target/release/mcp-utc-time-server | jq '.result | {unix: .unix.seconds, iso8601}'
```

### Custom Format
```bash
echo '{"jsonrpc":"2.0","method":"time/get_with_format","params":{"format":"%Y-%m-%d %H:%M:%S"},"id":1}' | \
  ./target/release/mcp-utc-time-server | jq -r '.result.formatted'
```

### Timezone Conversion
```bash
echo '{"jsonrpc":"2.0","method":"time/get_with_timezone","params":{"timezone":"Asia/Tokyo"},"id":1}' | \
  ./target/release/mcp-utc-time-server | jq '.result | {timezone, iso8601, offset}'
```

## Verification

Run all tests:

```bash
# Unit + integration tests
cargo test --all

# VSCode integration test
./scripts/test_vscode.sh

# MCPO integration test (if MCPO running)
export MCPO_API_KEY="your-key"
./scripts/test_mcpo.sh

# Full demo
./examples/demo.sh
```

## Troubleshooting

**Server won't start:**
```bash
# Check binary exists
ls -l target/release/mcp-utc-time-server

# Rebuild if needed
cargo clean && cargo build --release
```

**VSCode can't find server:**
- Use absolute path in settings.json
- Check PATH includes Rust binaries
- Verify with: `which mcp-utc-time-server`

**MCPO not working:**
```bash
# Check if installed
mcpo --version
# or
uvx mcpo --help

# Verify port is free
lsof -i :8000
```

## Next Steps

- Read full documentation: [README.md](README.md)
- Integration guide: [docs/INTEGRATION.md](docs/INTEGRATION.md)
- Implementation details: [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- Run benchmarks: `cargo bench`

## Support

- Issues: GitHub Issues
- Documentation: `/docs` directory
- Examples: `/examples` and `/scripts` directories

---

**Ready to use!** The server is production-ready with full test coverage.
