# Azure Container Apps Deployment

## Prerequisites

1. Azure CLI installed and logged in:
```bash
az login
az account set --subscription <subscription-id>
```

2. Container Registry setup:
```bash
az acr create \
  --resource-group mcp-time-rg \
  --name mcptimeregistry \
  --sku Basic

az acr login --name mcptimeregistry
```

## Build and Push Image

```bash
# Build the image
docker build -f Dockerfile.hardware -t mcp-utc-time-server:latest .

# Tag for ACR
docker tag mcp-utc-time-server:latest mcptimeregistry.azurecr.io/mcp-utc-time-server:latest

# Push to registry
docker push mcptimeregistry.azurecr.io/mcp-utc-time-server:latest
```

## Deploy to Azure Container Apps

```bash
# Create resource group
az group create \
  --name mcp-time-rg \
  --location eastus

# Create Container Apps environment
az containerapp env create \
  --name mcp-time-env \
  --resource-group mcp-time-rg \
  --location eastus

# Create Container App
az containerapp create \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --environment mcp-time-env \
  --image mcptimeregistry.azurecr.io/mcp-utc-time-server:latest \
  --target-port 3000 \
  --ingress external \
  --registry-server mcptimeregistry.azurecr.io \
  --registry-username <registry-username> \
  --registry-password <registry-password> \
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
