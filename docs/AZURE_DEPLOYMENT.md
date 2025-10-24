# Azure Container Apps Deployment

## Prerequisites

1. Azure CLI installed and logged in:
```bash
az login
az account set --subscription <subscription-id>
```

2. Container images are published to GitHub Container Registry (GHCR) automatically via GitHub Actions
   - No separate container registry setup needed
   - Images are available at: `ghcr.io/arressjay/mcp-utc-time-server:latest`

# Build and Push Image

**Images are automatically built and published to GitHub Container Registry (GHCR) via GitHub Actions.**

The workflow `.github/workflows/ghcr-publish.yml` automatically:
- Runs tests
- Builds the Docker image
- Pushes to GHCR on every push to `main` or `release/**` branches

**Available images:**
```bash
# Latest from main branch
ghcr.io/arressjay/mcp-utc-time-server:latest

# Specific release tags
ghcr.io/arressjay/mcp-utc-time-server:v1.0.0
```

**To trigger a new build:**
```bash
# Commit and push to main or release branch
git add .
git commit -m "Update deployment"
git push origin release/v1.0

# Watch the build
gh run watch
```

**Manual build (if needed):**
```bash
# Build locally
docker build -f Dockerfile -t mcp-utc-time-server:latest .

# Tag for GHCR
docker tag mcp-utc-time-server:latest ghcr.io/arressjay/mcp-utc-time-server:latest

# Login and push
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
docker push ghcr.io/arressjay/mcp-utc-time-server:latest
```

## Deploy to Azure Container Apps

```bash
# Create resource group (example uses australiasoutheast)
az group create \
  --name mcp-time-rg \
  --location australiasoutheast

# Create Container Apps environment
az containerapp env create \
  --name mcp-time-env \
  --resource-group mcp-time-rg \
  --location eastus

# Create Container App using GHCR image
# Note: GHCR public images don't require registry credentials
az containerapp create \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --environment mcp-time-env \
  --image ghcr.io/arressjay/mcp-utc-time-server:latest \
  --target-port 3000 \
  --ingress external \
  --env-vars \
    "API_KEY_1=secretvalue(api-key-1)" \
    "API_KEY_2=secretvalue(api-key-2)" \
    "NTP_SERVERS=time.cloudflare.com,time.google.com" \
    "LOCAL_STRATUM=10" \
    "ENABLE_PPS=no" \
    "ENABLE_GPS=no" \
  --cpu 0.5 \
  --memory 1Gi \
  --min-replicas 1 \
  --max-replicas 3
```

## Using Secrets

```bash
# Create secrets for API keys
az containerapp secret set \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --secrets \
    api-key-1=<your-secret-key-1> \
    api-key-2=<your-secret-key-2>

# Update environment variables to use secrets
az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --set-env-vars \
    "API_KEY_1=secretref:api-key-1" \
    "API_KEY_2=secretref:api-key-2"

## NTP / Time synchronization guidance for Azure deployments

### Shared Memory Interface (SHM) Configuration

The MCP UTC Time Server uses NTPsec's shared memory driver for nanosecond-precision timing (<1µs latency). This requires NTPsec running inside the container with proper configuration:

**Container Requirements:**
- NTPsec package installed (already in `Dockerfile.local`)
- Shared memory segment accessible (key: `0x4e545030+unit`)
- NTPsec daemon running with SHM driver configured

**NTP Configuration for SHM:**
The server expects NTPsec to provide time via shared memory segment unit 0. The included `config/ntp.conf.template` provides:

```conf
# Shared Memory Driver (type 28) - unit 0
server 127.127.28.0 mode 1 prefer
fudge 127.127.28.0 refid SHM0
```

**Starting NTPsec in Container:**
```bash
# Copy NTP config
cp /etc/ntpsec/ntp.conf.template /etc/ntp.conf

# Create required directories
mkdir -p /var/lib/ntpsec /var/log/ntpsec

# Set permissions for shared memory (0666 for IPC)
# NTPsec will create SHM segment automatically

# Start NTPsec daemon
ntpd -c /etc/ntp.conf -g
```

**Health Check Verification:**
The `/health` endpoint reports SHM interface status:
```json
{
  "ntp": {
    "shm_valid": true,
    "pps_enabled": false,
    "shm_interface": "active",
    "hardware_clock": "Not detected"
  }
}
```

Notes:
- Microsoft recommends using the Azure host time or Microsoft-maintained NTP sources for most workloads. For Container Apps (PaaS), the platform manages host time; however if your application needs to query NTP servers (for diagnostics or monitoring) you may configure `NTP_SERVERS` env var to a short list of regional NTP servers.
- The SHM interface provides <1µs latency vs ~10ms for shell commands
- For hardware clock support (GPIO PPS), see `docs/NTP_SHM_SETUP.md`
- For `australiasoutheast` choose regionally close, reliable public NTP servers. Examples:
  - time.cloudflare.com (global Anycast)
  - time.google.com (global Anycast)
  - Your national metrology or NTP pools (e.g., au.pool.ntp.org)

Recommended process:
1. Prefer the Azure host time when possible (reads from the platform). See: https://learn.microsoft.com/en-us/azure/virtual-machines/linux/time-sync
2. If you must use external NTP servers (NTPsec inside container), pick 2–4 regionally close servers, e.g. `au.pool.ntp.org`, `time.cloudflare.com`, and `time.google.com`.
3. Configure NTPsec inside your container to use those servers and use conservative polling (do not overload public servers). Use `LOCAL_STRATUM` to set how your service advertises itself if needed.
4. For strict time requirements (financial services, authentication), consider using a dedicated, internal NTP infrastructure (Key Vault for secrets and internal NTP endpoints) and use Azure VMs or edge devices with PPS/GPS if you need Stratum-1 sources (see RASPBERRY_PI.md for edge details).

Relevant Microsoft documentation:
- Time sync for Linux VMs: https://learn.microsoft.com/en-us/azure/virtual-machines/linux/time-sync
- Time sync for Windows VMs: https://learn.microsoft.com/en-us/azure/virtual-machines/windows/time-sync
- Azure security benchmark: Use approved time synchronization sources: https://learn.microsoft.com/en-us/security/benchmark/azure/mcsb-logging-threat-detection#lt-7-use-approved-time-synchronization-sources

```

## Health Checks

```bash
az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --health-probe-type liveness \
  --health-probe-path /health \
  --health-probe-interval 30 \
  --health-probe-timeout 5 \
  --health-probe-retries 3
```

## Scaling Configuration

```bash
# Scale based on HTTP requests
az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --scale-rule-name http-rule \
  --scale-rule-type http \
  --scale-rule-metadata concurrentRequests=10 \
  --min-replicas 1 \
  --max-replicas 10
```

## Monitoring

```bash
# Enable Application Insights
az monitor app-insights component create \
  --app mcp-time-insights \
  --location eastus \
  --resource-group mcp-time-rg

# Link to Container App
INSTRUMENTATION_KEY=$(az monitor app-insights component show \
  --app mcp-time-insights \
  --resource-group mcp-time-rg \
  --query instrumentationKey -o tsv)

az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --set-env-vars "APPLICATIONINSIGHTS_CONNECTION_STRING=InstrumentationKey=$INSTRUMENTATION_KEY"
```

## View Logs

```bash
# Stream logs
az containerapp logs show \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --follow

# Query logs
az containerapp logs show \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --type console
```

## Cleanup

```bash
# Delete Container App
az containerapp delete \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --yes

# Delete entire resource group
az group delete \
  --name mcp-time-rg \
  --yes
```

## Custom Domain (Optional)

```bash
# Add custom domain
az containerapp hostname add \
  --hostname time-api.example.com \
  --name mcp-utc-time \
  --resource-group mcp-time-rg

# Bind certificate
az containerapp hostname bind \
  --hostname time-api.example.com \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --certificate <certificate-id> \
  --environment mcp-time-env
```

## Using with MCP Client

Once deployed, connect to the service using:

```json
{
  "mcpServers": {
    "utc-time": {
      "url": "https://mcp-utc-time.<unique-id>.eastus.azurecontainerapps.io",
      "headers": {
        "X-API-Key": "<your-api-key>"
      }
    }
  }
}
```
