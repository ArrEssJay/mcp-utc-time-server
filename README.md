# MCP UTC Time Server

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue.svg)](https://modelcontextprotocol.io)
[![Tests](https://img.shields.io/badge/tests-31%20passing-brightgreen.svg)](./docs/TEST_REPORT.md)
[![Performance](https://img.shields.io/badge/latency-18ns-success.svg)](./docs/PERFORMANCE.md)

**Production-ready time server for AI agents with nanosecond precision, 595+ timezones, and NTP hardware clock support.**

Built on the Model Context Protocol (MCP), this server provides AI agents, VSCode extensions, and web applications with forensically accurate timestamps, timezone conversions, and time formatting—all with sub-microsecond performance.

🌐 **Live API**: <https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io>

---

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/mcp-utc-time-server

# Test HTTP API
curl http://localhost:3000/api/time | jq .

# Test MCP protocol
./scripts/test_mcp_protocol.sh
```

**For VSCode setup**: See [QUICKSTART.md](./QUICKSTART.md)

---

## Why This Server?

### 🚀 Built for AI Agents

- **9 MCP Tools**: Time queries, timezone conversions, NTP status
- **4 MCP Prompts**: `/time`, `/unix_time`, `/time_in`, `/format_time`
- **Full MCP 2024-11-05 compliance** with tools, prompts, and proper handshaking

### ⚡ Exceptional Performance

Real benchmarks on Apple Silicon:

- **18.4 ns** per Unix timestamp (54M ops/sec)
- **148 ns** per formatted timestamp (6.7M ops/sec)  
- **1.16 μs** per complete response (862K ops/sec)

See [PERFORMANCE.md](./docs/PERFORMANCE.md) for detailed analysis.

### 🌍 Complete Timezone Support

- **595+ IANA timezones** with automatic DST handling
- **Nanosecond precision** (9 decimal places)
- **Multiple formats**: ISO 8601, RFC 3339, RFC 2822, Unix, Syslog, Apache
- **Custom strftime formatting** for any output format

### 🔒 Production Ready

- ✅ **Deployed on Azure** Container Apps (live endpoint)
- ✅ **31 tests** with 100% pass rate, zero Clippy warnings
- ✅ **Docker + Kubernetes** ready
- ✅ **Raspberry Pi support** with GPS/PPS hardware clocks
- ✅ **Comprehensive documentation** for all deployment scenarios

---

## Installation

### From Source

```bash
git clone https://github.com/ArrEssJay/mcp-utc-time-server
cd mcp-utc-time-server
cargo build --release
```

### VSCode Configuration

Add to `.vscode/settings.json` or user settings:

```json
{
  "mcp.servers": {
    "utc-time": {
      "command": "/path/to/mcp-utc-time-server/target/release/mcp-utc-time-server"
    }
  }
}
```

Then reload VSCode or run "MCP: Reload Servers".

### Docker

```bash
docker build -t mcp-utc-time-server .
docker run -p 3000:3000 mcp-utc-time-server
```

---

## Usage

### MCP Tools (for AI Agents)

All tools are automatically discovered by MCP-compatible AI agents:

| Tool | Description | Arguments |
|------|-------------|-----------|
| `get_time` | Complete time data with all formats | None |
| `get_unix_time` | Unix timestamp with nanoseconds | None |
| `get_nanos` | Nanoseconds since Unix epoch | None |
| `get_time_formatted` | Custom strftime format | `format` (string) |
| `get_time_with_timezone` | Time in specific timezone | `timezone` (IANA name) |
| `list_timezones` | All 595+ available timezones | None |
| `convert_time` | Convert between timezones | `timestamp`, `to_timezone` |
| `get_ntp_status` | NTP synchronization status | None |
| `get_ntp_peers` | NTP peer information | None |

### MCP Prompts (for Users)

Slash commands available in VSCode:

- `/time` - Get current UTC time
- `/unix_time` - Get Unix timestamp
- `/time_in <timezone>` - Get time in specific timezone  
- `/format_time <format>` - Custom formatted time

### HTTP API

**Base URL**: `https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io`

```bash
# Health check
curl $BASE/health

# Current time (all formats)
curl $BASE/api/time | jq .

# Unix timestamp
curl $BASE/api/unix

# Time in timezone
curl $BASE/api/time/timezone/America/New_York

# List timezones
curl $BASE/api/timezones

# NTP status
curl $BASE/api/ntp/status

# Prometheus metrics
curl $BASE/metrics
```

Full API documentation: [HTTP_API.md](./docs/HTTP_API.md) | [OpenAPI Spec](./openapi.yaml)

---

## Example Response

```json
{
  "unix": {
    "seconds": 1761285264,
    "nanos": 680042000,
    "nanos_since_epoch": 1761285264680042000
  },
  "iso8601": "2025-10-24T05:54:24.680042000Z",
  "rfc3339": "2025-10-24T05:54:24.680042+00:00",
  "rfc2822": "Fri, 24 Oct 2025 05:54:24 +0000",
  "year": 2025,
  "month": 10,
  "day": 24,
  "hour": 5,
  "minute": 54,
  "second": 24,
  "nanosecond": 680042000,
  "timezone": "UTC",
  "weekday": "Friday",
  "week_of_year": 42
}
```

---

## Deployment Options

### Local Development

```bash
cargo run --release
# Server starts on stdio (MCP) and HTTP on port 3000
```

### Azure Container Apps

Complete Infrastructure as Code included:

```bash
cd infra
./deploy.sh
```

See [AZURE_DEPLOYMENT.md](./docs/AZURE_DEPLOYMENT.md)

### Kubernetes

```bash
kubectl apply -f k8s/deployment.yaml
```

### Raspberry Pi (with Hardware Clock)

For GPS/PPS-synchronized time:

```bash
./scripts/configure-ntp.sh
cargo build --release
./target/release/mcp-utc-time-server
```

See [RASPBERRY_PI.md](./docs/RASPBERRY_PI.md) | [NTP_SHM_SETUP.md](./docs/NTP_SHM_SETUP.md)

### ChatGPT Custom GPT

Expose via Cloudflare Tunnel:

```bash
export MCPO_API_KEY="$(openssl rand -base64 32)"
./scripts/start_tunnel.sh
```

See [TUNNEL_SETUP.md](./docs/TUNNEL_SETUP.md)

---

## Testing

```bash
# All tests (unit + integration)
cargo test --all

# MCP protocol compliance
./scripts/test_mcp_protocol.sh

# HTTP API
./scripts/test_http_api.sh

# Performance benchmarks
cargo bench
```

**Test Results**: 31/31 passing (100%)
- 10 unit tests
- 12 HTTP API tests
- 9 integration tests

See [TEST_REPORT.md](./docs/TEST_REPORT.md)

---

## Documentation

- **[QUICKSTART.md](./QUICKSTART.md)** - 5-minute setup guide
- **[MCP_COMPLIANCE.md](./docs/MCP_COMPLIANCE.md)** - Protocol implementation details
- **[HTTP_API.md](./docs/HTTP_API.md)** - REST API guide
- **[INTEGRATION.md](./docs/INTEGRATION.md)** - VSCode, MCPO, ChatGPT setup
- **[PERFORMANCE.md](./docs/PERFORMANCE.md)** - Benchmark analysis
- **[AZURE_DEPLOYMENT.md](./docs/AZURE_DEPLOYMENT.md)** - Cloud deployment
- **[RASPBERRY_PI.md](./docs/RASPBERRY_PI.md)** - Hardware clock setup
- **[TEST_REPORT.md](./docs/TEST_REPORT.md)** - Test coverage details

---

## Architecture

```
mcp-utc-time-server/
├── src/
│   ├── server_sdk.rs      # MCP server implementation (rmcp SDK)
│   ├── time/              # Time operations
│   │   ├── utc.rs         # Enhanced time response
│   │   ├── unix.rs        # Unix time with nanoseconds
│   │   ├── formats.rs     # Strftime formatting
│   │   └── timezone.rs    # Timezone conversions
│   ├── ntp/               # NTP synchronization
│   │   ├── sync.rs        # SHM interface
│   │   └── config.rs      # Configuration
│   └── auth/              # API key authentication
├── docs/                  # Documentation
├── scripts/               # Helper scripts
├── infra/                 # Azure Bicep templates
└── k8s/                   # Kubernetes manifests
```

**Key Technologies**:

- **MCP SDK**: Official `rmcp` v0.8 with macros
- **Async Runtime**: Tokio for high performance
- **Time Handling**: Chrono + chrono-tz for timezone support
- **NTP Integration**: Custom SHM interface for hardware clocks
- **Serialization**: Serde with JSON-RPC 2.0

---

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality  
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

---

## License

MIT License - see [LICENSE](LICENSE)

---

## Support

- **GitHub Issues**: <https://github.com/ArrEssJay/mcp-utc-time-server/issues>
- **Documentation**: <https://github.com/ArrEssJay/mcp-utc-time-server/tree/main/docs>
- **Live API**: <https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/health>

---

## Acknowledgments

Built with:

- [rmcp](https://github.com/modelcontextprotocol/rmcp) - Official Rust MCP SDK
- [Tokio](https://tokio.rs/) - Async runtime
- [Chrono](https://github.com/chronotope/chrono) - Time handling
- [Chrono-TZ](https://github.com/chronotope/chrono-tz) - Timezone database
- [Serde](https://serde.rs/) - Serialization

Inspired by the [Model Context Protocol](https://modelcontextprotocol.io) specification.
