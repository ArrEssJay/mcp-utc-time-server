# Infrastructure Deployment

This folder contains Bicep templates for deploying the MCP UTC Time Server to Azure Container Apps.

## Container Registry

**Images are hosted on GitHub Container Registry (GHCR):**
- `ghcr.io/arressjay/mcp-utc-time-server:latest`
- Built automatically via `.github/workflows/ghcr-publish.yml`

## Quick Deploy

```bash
# 1. Trigger a build by pushing to main or release branch
git push origin release/v1.0

# 2. Watch the build complete
gh run watch

# 3. Deploy infrastructure
az deployment group create \
  --resource-group mcp-time-rg \
  --template-file infra/main.bicep \
  --parameters @infra/main.parameters.json

# 4. Deploy/update the container app
az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --image ghcr.io/arressjay/mcp-utc-time-server:latest
```

## Preview Changes (What-If)

```bash
# Validate with a what-if preview (no changes applied)
az deployment group what-if \
  --resource-group mcp-time-rg \
  --template-file infra/main.bicep \
  --parameters @infra/main.parameters.json
```

## Full Deployment

```bash
az deployment group create \
  --resource-group mcp-time-rg \
  --template-file infra/main.bicep \
  --parameters @infra/main.parameters.json
```

Notes:

- The template creates Container Apps environment, Key Vault, and Log Analytics workspace
- Container images are pulled from GHCR (no ACR needed)
- Secrets should be created in Key Vault via a secure pipeline or administrator action
- Do not enable purge protection for Key Vault unless you understand the operational implications
