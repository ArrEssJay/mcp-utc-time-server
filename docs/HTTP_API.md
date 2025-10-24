# HTTP API Documentation

## Overview

The MCP UTC Time Server provides a REST API for agents and applications that cannot use the MCP stdio protocol directly. This is ideal for:

- Web applications and services
- AI agents (ChatGPT, Claude, etc.) via HTTP
- Monitoring systems (Prometheus, Grafana)
- Remote integrations
- Containerized deployments

**Production URL**: `https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io`

## OpenAPI Specification

The complete API specification is available in **OpenAPI 3.1** format:

📄 **[openapi.yaml](../openapi.yaml)**

### Using the OpenAPI Spec

#### 1. **AI Agents (ChatGPT, Claude)**

AI agents can consume the OpenAPI spec to understand and use the API:

**ChatGPT**:
```
I have an API with OpenAPI spec at:
https://raw.githubusercontent.com/ArrEssJay/mcp-utc-time-server/main/openapi.yaml

Please analyze this API and help me use it.
```

**Claude** (with Claude Desktop):
- Add to MCP config with HTTP transport (when available)
- Or use via manual curl commands

#### 2. **API Documentation Viewers**

View interactive documentation:

- **Swagger UI**: https://editor.swagger.io/
  - Paste openapi.yaml content
  - Try live API calls
  
- **Redoc**: https://redocly.github.io/redoc/
  - Beautiful, responsive docs
  
- **Postman**: Import openapi.yaml
  - Create collection from OpenAPI
  - Test all endpoints

#### 3. **Code Generation**

Generate client SDKs:

```bash
# Install OpenAPI Generator
npm install -g @openapitools/openapi-generator-cli

# Generate Python client
openapi-generator-cli generate \
  -i openapi.yaml \
  -g python \
  -o ./clients/python

# Generate JavaScript client
openapi-generator-cli generate \
  -i openapi.yaml \
  -g javascript \
  -o ./clients/javascript

# Generate Go client
openapi-generator-cli generate \
  -i openapi.yaml \
  -g go \
  -o ./clients/go
```

## Quick Start Examples

### Health Check

```bash
curl https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/health
```

Response:
```json
{
  "status": "healthy",
  "service": "mcp-utc-time-server",
  "version": "0.1.0",
  "timestamp": "2025-10-24T05:06:22.515919331+00:00",
  "ntp": {
    "synced": true,
    "stratum": 3,
    "offset_ms": 0.0,
    "shm_valid": false,
    "pps_enabled": false
  }
}
```

### Get Current Time

```bash
curl https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/api/time | jq
```

### Get Unix Timestamp

```bash
curl https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/api/unix
```

Response:
```json
{
  "seconds": 1761282399,
  "nanos": 509856152,
  "nanos_since_epoch": 1761282399509856152
}
```

### Get Time in Timezone

```bash
curl https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/api/time/timezone/America/New_York
```

### List All Timezones

```bash
curl https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io/api/timezones
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check with NTP status |
| `/metrics` | GET | Prometheus metrics |
| `/api/time` | GET | Complete time information |
| `/api/unix` | GET | Unix timestamp (seconds + nanos) |
| `/api/nanos` | GET | Nanoseconds since epoch |
| `/api/timezones` | GET | List all IANA timezones |
| `/api/time/timezone/{tz}` | GET | Time in specific timezone |
| `/api/ntp/status` | GET | NTP synchronization status |

## CORS Support

All endpoints support CORS with permissive headers:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

This allows web applications to call the API directly from browsers.

## Response Formats

All responses are JSON except `/metrics` (Prometheus text format).

### Standard Fields

Every time response includes:
- **Unix timestamps**: seconds, milliseconds, microseconds, nanoseconds
- **Standard formats**: ISO8601, RFC3339, RFC2822, ctime
- **Components**: year, month, day, hour, minute, second, nanosecond
- **Metadata**: timezone, offset, weekday, week_of_year, day_of_year

### Error Responses

```json
{
  "error": "Error type",
  "message": "Detailed error description"
}
```

## Performance

- **Latency**: ~1-5ms (Azure Australia Southeast)
- **Precision**: Sub-microsecond timestamps
- **Rate Limit**: None (but subject to Azure Container Apps limits)
- **Availability**: 99.9% SLA (Azure Container Apps)

## Monitoring Integration

### Prometheus

Add to prometheus.yml:

```yaml
scrape_configs:
  - job_name: 'mcp-time'
    scrape_interval: 30s
    static_configs:
      - targets: ['mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io']
    metrics_path: '/metrics'
    scheme: https
```

### Grafana

Create dashboard with queries:
- `mcp_time_seconds` - Current Unix timestamp
- `mcp_time_nanos` - Nanosecond component

## Agent Integration Examples

### ChatGPT Custom Action

Create a GPT with this schema:

```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "Time API",
    "version": "1.0.0"
  },
  "servers": [
    {
      "url": "https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io"
    }
  ],
  "paths": {
    "/api/time": {
      "get": {
        "operationId": "get_time",
        "summary": "Get current UTC time"
      }
    }
  }
}
```

Or reference the full openapi.yaml URL:
```
https://raw.githubusercontent.com/ArrEssJay/mcp-utc-time-server/main/openapi.yaml
```

### Python Agent

```python
import requests

class TimeAPI:
    def __init__(self, base_url="https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io"):
        self.base_url = base_url
    
    def get_time(self):
        """Get current UTC time"""
        response = requests.get(f"{self.base_url}/api/time")
        return response.json()
    
    def get_unix(self):
        """Get Unix timestamp"""
        response = requests.get(f"{self.base_url}/api/unix")
        return response.json()
    
    def get_timezone(self, tz):
        """Get time in specific timezone"""
        response = requests.get(f"{self.base_url}/api/time/timezone/{tz}")
        return response.json()

# Usage
api = TimeAPI()
print(api.get_time())
print(api.get_timezone("America/New_York"))
```

### JavaScript/Node.js Agent

```javascript
class TimeAPI {
  constructor(baseUrl = 'https://mcp-utc-time.bluedune-ec819a83.australiasoutheast.azurecontainerapps.io') {
    this.baseUrl = baseUrl;
  }

  async getTime() {
    const response = await fetch(`${this.baseUrl}/api/time`);
    return response.json();
  }

  async getUnix() {
    const response = await fetch(`${this.baseUrl}/api/unix`);
    return response.json();
  }

  async getTimezone(tz) {
    const response = await fetch(`${this.baseUrl}/api/time/timezone/${tz}`);
    return response.json();
  }
}

// Usage
const api = new TimeAPI();
const time = await api.getTime();
console.log(time);
```

## Local Development

Run locally on port 3000:

```bash
# Build and run
cargo run

# Or run the release binary
./target/release/mcp-utc-time-server
```

Test locally:

```bash
curl http://localhost:3000/health
curl http://localhost:3000/api/time
```

## Container Deployment

The HTTP API runs automatically in container mode:

```bash
# Docker
docker run -p 3000:3000 ghcr.io/arressjay/mcp-utc-time-server:latest

# Test
curl http://localhost:3000/health
```

Environment variables:
- `HTTP_API_PORT` or `HEALTH_PORT`: Port to listen on (default: 3000)
- `RUST_LOG`: Log level (default: info)
- `CONTAINER_APP_NAME`: Automatically set by Azure Container Apps

## Testing

Run HTTP API tests:

```bash
# Automated test suite
./scripts/test_http_api.sh

# Or with cargo
cargo test --test http_api_test
```

See [HTTP_API_TESTING.md](HTTP_API_TESTING.md) for details.

## Support

- **Issues**: https://github.com/ArrEssJay/mcp-utc-time-server/issues
- **Documentation**: https://github.com/ArrEssJay/mcp-utc-time-server/tree/main/docs
- **OpenAPI Spec**: https://github.com/ArrEssJay/mcp-utc-time-server/blob/main/openapi.yaml

## See Also

- [INTEGRATION.md](INTEGRATION.md) - MCP stdio integration
- [HTTP_API_TESTING.md](HTTP_API_TESTING.md) - Testing guide
- [AZURE_DEPLOYMENT.md](AZURE_DEPLOYMENT.md) - Deployment guide
