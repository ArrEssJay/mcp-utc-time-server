# GitHub Actions Workflows

This directory contains CI/CD workflows for the MCP UTC Time Server.

## Workflows

### `ghcr-publish.yml` - Build, Test, and Deploy

**Trigger**: Push to `main` or `release/**` branches

**Jobs**:

1. **test** - Runs all tests, formatting checks, and linting
2. **build-and-publish** - Builds Docker image and pushes to GitHub Container Registry (GHCR)
3. **deploy-to-azure** - Automatically deploys the new image to Azure Container Apps

## Setup

### One-Time Azure Setup

To enable automatic deployment to Azure, run:

```bash
# Make sure you're logged into Azure and have gh CLI installed
az login
gh auth login

# Run the setup script
./.github/setup-azure-deploy.sh
```

This script will:
- Create an Azure service principal with Contributor access to the `mcp-time-rg` resource group
- Configure the following GitHub secrets:
  - `AZURE_CLIENT_ID`
  - `AZURE_CLIENT_SECRET`
  - `AZURE_TENANT_ID`
  - `AZURE_SUBSCRIPTION_ID`

### Manual Deployment

If you need to deploy manually without GitHub Actions:

```bash
# Build and push locally
docker build -t ghcr.io/arressjay/mcp-utc-time-server:latest .
docker push ghcr.io/arressjay/mcp-utc-time-server:latest

# Deploy to Azure
az containerapp update \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --image ghcr.io/arressjay/mcp-utc-time-server:latest
```

## Deployment Flow

```
┌─────────────────┐
│ Push to release │
│   or main       │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   Run Tests     │
│  - cargo test   │
│  - clippy       │
│  - rustfmt      │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Build & Push    │
│   to GHCR       │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Deploy to Azure │
│  Container Apps │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   🎉 Live!      │
└─────────────────┘
```

## Monitoring Deployments

View workflow runs:
```bash
gh run list
```

Watch a running workflow:
```bash
gh run watch
```

View logs for a specific run:
```bash
gh run view <run-id> --log
```

## Troubleshooting

### Deployment Fails

Check the workflow logs:
```bash
gh run list --workflow="Build, Test, and Publish to GHCR"
gh run view <run-id> --log
```

### Check Azure Container App Status

```bash
# Check revision health
az containerapp revision list \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --query "[].{Name:name, Active:properties.active, Health:properties.healthState}" \
  -o table

# View logs
az containerapp logs show \
  --name mcp-utc-time \
  --resource-group mcp-time-rg \
  --tail 50
```

### Secrets Expired or Invalid

Re-run the setup script:
```bash
./.github/setup-azure-deploy.sh
```
