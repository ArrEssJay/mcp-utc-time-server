# Secrets & Key Vault guidance

This document explains how to store API keys and other secrets securely for production deployments in Azure.

Recommended approach

1. Create an Azure Key Vault in the target resource group (australiasoutheast).
2. Create secrets in Key Vault for each API key (e.g. `api-key-1`, `api-key-2`, etc.).
3. Create a user-assigned managed identity and grant it `Key Vault Secrets User` role (or use Key Vault access policies / RBAC) so the Container App can retrieve secrets at runtime.
4. Configure Container App to use the managed identity and reference secrets using `secretref:` or via Key Vault retrieval in your startup code.

Example commands

```bash
# Create Key Vault
az keyvault create --name my-mcp-kv --resource-group mcp-time-rg --location australiasoutheast

# Create secrets
az keyvault secret set --vault-name my-mcp-kv --name api-key-1 --value "<your-key-1>"
az keyvault secret set --vault-name my-mcp-kv --name api-key-2 --value "<your-key-2>"

# Create user-assigned managed identity
az identity create -g mcp-time-rg -n mcp-identity

# Grant the managed identity permission to get secrets
IDENTITY_PRINCIPAL_ID=$(az identity show -g mcp-time-rg -n mcp-identity --query principalId -o tsv)
az keyvault set-policy -n my-mcp-kv --object-id $IDENTITY_PRINCIPAL_ID --secret-permissions get list

# In Container Apps, assign the identity to the app and reference secrets either by
# creating Container App secrets (pulling from Key Vault in CI) or retrieving them
# from Key Vault in the application code using DefaultAzureCredential.
```

Rotation & audit

- Rotate keys regularly and update Key Vault secrets. Use Key Vault soft-delete + purge protection to avoid accidental loss.
- Use Key Vault audit logs (Azure Monitor) to track access to secrets.
