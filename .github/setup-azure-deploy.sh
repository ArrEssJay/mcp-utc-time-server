#!/usr/bin/env bash
# Setup Azure deployment credentials for GitHub Actions
# This script creates a service principal with permissions to update the Container App
# and configures GitHub secrets for automated deployment

set -euo pipefail

RESOURCE_GROUP="mcp-time-rg"
APP_NAME="mcp-utc-time"
SP_NAME="github-actions-mcp-time"

echo "🔧 Setting up Azure deployment for GitHub Actions..."
echo ""

# Get subscription ID
SUBSCRIPTION_ID=$(az account show --query id -o tsv)
echo "✓ Using subscription: $SUBSCRIPTION_ID"

# Get resource group ID
RG_ID=$(az group show --name "$RESOURCE_GROUP" --query id -o tsv)
echo "✓ Found resource group: $RESOURCE_GROUP"

# Create service principal with Contributor role on the resource group
echo ""
echo "Creating service principal..."
SP_OUTPUT=$(az ad sp create-for-rbac \
  --name "$SP_NAME" \
  --role Contributor \
  --scopes "$RG_ID" \
  --sdk-auth)

# Extract values
CLIENT_ID=$(echo "$SP_OUTPUT" | jq -r '.clientId')
CLIENT_SECRET=$(echo "$SP_OUTPUT" | jq -r '.clientSecret')
TENANT_ID=$(echo "$SP_OUTPUT" | jq -r '.tenantId')

echo "✓ Service principal created: $CLIENT_ID"
echo ""

# Configure GitHub secrets using gh CLI
echo "📝 Configuring GitHub secrets..."
echo ""

gh secret set AZURE_CLIENT_ID --body "$CLIENT_ID"
echo "✓ Set AZURE_CLIENT_ID"

gh secret set AZURE_CLIENT_SECRET --body "$CLIENT_SECRET"
echo "✓ Set AZURE_CLIENT_SECRET"

gh secret set AZURE_TENANT_ID --body "$TENANT_ID"
echo "✓ Set AZURE_TENANT_ID"

gh secret set AZURE_SUBSCRIPTION_ID --body "$SUBSCRIPTION_ID"
echo "✓ Set AZURE_SUBSCRIPTION_ID"

echo ""
echo "✅ Azure deployment setup complete!"
echo ""
echo "The following GitHub secrets have been configured:"
echo "  - AZURE_CLIENT_ID"
echo "  - AZURE_CLIENT_SECRET"
echo "  - AZURE_TENANT_ID"
echo "  - AZURE_SUBSCRIPTION_ID"
echo ""
echo "The service principal '$SP_NAME' has Contributor access to resource group '$RESOURCE_GROUP'"
echo ""
echo "Next push to 'main' or 'release/**' branches will automatically:"
echo "  1. Run tests"
echo "  2. Build and push Docker image to GHCR"
echo "  3. Deploy to Azure Container Apps"
