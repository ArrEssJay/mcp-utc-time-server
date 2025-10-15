# MCP UTC Time Server

A high-precision time server implementing the Model Context Protocol (MCP) for AI agents in VSCode, with full Unix/POSIX time support including nanosecond precision, strftime formatting, and comprehensive timezone handling.

## Features

### 🕐 Nanosecond Precision
- Full nanosecond precision Unix timestamps
- Support for seconds, milliseconds, microseconds, and nanoseconds since epoch
- Compatible with `libc::timespec` for POSIX systems

### 📅 Multiple Time Formats
- **ISO 8601**: Full RFC 3339 compliance with nanosecond precision
- **RFC 2822**: Email and HTTP header format
- **Unix Date**: Standard Unix `date` command format
- **Syslog**: Standard syslog timestamp format
- **Apache Log**: Common web server log format
- **Custom strftime**: Full C-style strftime format support

### 🌍 Timezone Support
- Complete IANA timezone database (595+ timezones)
- Timezone conversion with offset calculation
- Support for DST-aware conversions
- POSIX TZ string parsing

### 🔌 MCP Protocol Integration
- JSON-RPC 2.0 compliant
- STDIO-based transport for VSCode integration
- Multiple time query methods
- Comprehensive error handling

## Installation

### Prerequisites
- Rust 1.75.0 or higher
- Cargo package manager

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-utc-time-server
cd mcp-utc-time-server

# Build release version
cargo build --release

# Run tests
cargo test --all

# Run benchmarks
cargo bench
```

### Install in VSCode

1. Add to your VSCode `settings.json`:

```json
{
    "mcp.servers": {
        "utc-time": {
            "command": "cargo",
            "args": ["run", "--release"],
            "cwd": "/path/to/mcp-utc-time-server",
            "env": {
                "RUST_LOG": "info",
                "TZ": "UTC"
            }
        }
    }
}
```

2. Reload VSCode or run the command: "MCP: Reload Servers"

## Usage

### Available Methods

#### `time/get` - Get Complete Time Information
Returns comprehensive time data including all formats and components.

```json
{
    "jsonrpc": "2.0",
    "method": "time/get",
    "params": {},
    "id": 1
}
```

**Response includes:**
- Unix time (seconds, nanos, microseconds, milliseconds)
- Standard formats (ISO 8601, RFC 3339, RFC 2822, ctime)
- Individual components (year, month, day, hour, minute, second, nanosecond)
- Week information (weekday, week of year, day of year)
- Custom format samples (unix_date, syslog, apache_log)

#### `time/get_unix` - Get Unix Timestamp
Returns Unix timestamp with nanosecond precision.

```json
{
    "jsonrpc": "2.0",
    "method": "time/get_unix",
    "params": {},
    "id": 2
}
```

#### `time/get_nanos` - Get Nanoseconds Since Epoch
Returns time in nanoseconds since Unix epoch.

```json
{
    "jsonrpc": "2.0",
    "method": "time/get_nanos",
    "params": {},
    "id": 3
}
```

#### `time/get_with_format` - Custom strftime Format
Format time using C-style strftime format strings.

```json
{
    "jsonrpc": "2.0",
    "method": "time/get_with_format",
    "params": {
        "format": "%Y-%m-%d %H:%M:%S.%f %Z"
    },
    "id": 4
}
```

**Supported Format Specifiers:**
- `%Y` - Year (e.g., 2024)
- `%m` - Month (01-12)
- `%d` - Day (01-31)
- `%H` - Hour (00-23)
- `%M` - Minute (00-59)
- `%S` - Second (00-60)
- `%f` - Microseconds
- `%z` - Timezone offset (+0000)
- `%Z` - Timezone name (UTC)
- `%s` - Unix timestamp
- `%c` - Complete date/time
- And many more...

#### `time/get_with_timezone` - Get Time in Specific Timezone
Convert current time to any IANA timezone.

```json
{
    "jsonrpc": "2.0",
    "method": "time/get_with_timezone",
    "params": {
        "timezone": "America/New_York"
    },
    "id": 5
}
```

#### `time/list_timezones` - List Available Timezones
Returns all 595+ IANA timezone identifiers.

```json
{
    "jsonrpc": "2.0",
    "method": "time/list_timezones",
    "params": {},
    "id": 6
}
```

#### `time/convert` - Convert Between Timezones
Convert a specific timestamp between timezones.

```json
{
    "jsonrpc": "2.0",
    "method": "time/convert",
    "params": {
        "timestamp": 1697472000,
        "from_timezone": "UTC",
        "to_timezone": "Asia/Tokyo"
    },
    "id": 7
}
```

## Examples

### Command Line Testing

```bash
# Start the server
cargo run --release

# In another terminal, test with echo and pipes
echo '{"jsonrpc":"2.0","method":"time/get","params":{},"id":1}' | cargo run --release

# Get Unix timestamp
echo '{"jsonrpc":"2.0","method":"time/get_unix","params":{},"id":1}' | cargo run --release

# Custom format
echo '{"jsonrpc":"2.0","method":"time/get_with_format","params":{"format":"%s"},"id":1}' | cargo run --release
```

### From VSCode/AI Agent

Once configured in VSCode, AI agents can call:

```
Get current UTC time with nanosecond precision
```

The MCP server will automatically handle the request and return comprehensive time information.

## Performance

**Actual Benchmark Results** (Apple Silicon M-series):
- Unix time generation: **18.4 ns/operation** (54M ops/sec)
- Custom formatting: **148 ns/operation** (6.7M ops/sec)
- Enhanced time response: **1.16 μs/operation** (862K ops/sec)

Memory usage: **<20 MB**

See [PERFORMANCE.md](docs/PERFORMANCE.md) for detailed benchmark analysis.

Run benchmarks: `cargo bench`

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test '*'
```

### With Coverage
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Development

### Project Structure
```
mcp-utc-time-server/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── error.rs          # Error types
│   ├── time/             # Time modules
│   │   ├── mod.rs
│   │   ├── utc.rs        # Enhanced UTC response
│   │   ├── unix.rs       # Unix time with nanos
│   │   ├── formats.rs    # strftime support
│   │   └── timezone.rs   # Timezone conversions
│   ├── mcp/              # MCP protocol
│   │   ├── mod.rs
│   │   ├── types.rs      # Protocol types
│   │   └── transport.rs  # STDIO transport
│   └── server/           # Server implementation
│       ├── mod.rs        # Server loop
│       ├── handlers.rs   # Request handlers
│       └── protocol.rs   # Protocol constants
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
└── .vscode/              # VSCode configuration
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## Troubleshooting

### Server Won't Start
- Check that port is not already in use
- Verify Rust toolchain is installed: `rustc --version`
- Try running with verbose logging: `RUST_LOG=debug cargo run`

### VSCode Can't Connect
- Verify MCP extension is installed
- Check `settings.json` configuration
- Ensure the path to the workspace folder is correct
- Check VSCode developer console for errors

### Time Drift Issues
- Ensure system clock is synchronised (use NTP)
- Check timezone environment variable: `echo $TZ`
- Verify system time: `date -u`

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Chrono](https://github.com/chronotope/chrono) - Time handling
- [Chrono-TZ](https://github.com/chronotope/chrono-tz) - Timezone database
- [Serde](https://serde.rs/) - Serialization

Inspired by the Model Context Protocol (MCP) specification.

## Support

For issues, questions, or contributions, please visit the [GitHub repository](https://github.com/arressjay/mcp-time-server).
