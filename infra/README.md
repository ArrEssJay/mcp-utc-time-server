# Infra deployment scaffold

This folder contains a starting Bicep template and parameter file for deploying the Container App environment, Key Vault, and Log Analytics workspace.

How to validate (preview only):

```bash
# Validate with a what-if preview (no changes applied)
az deployment group what-if \
  --resource-group <your-rg> \
  --template-file infra/main.bicep \
  --parameters @infra/main.parameters.json
```

To actually deploy (review the what-if first):

```bash
az deployment group create \
  --resource-group <your-rg> \
  --template-file infra/main.bicep \
  --parameters @infra/main.parameters.json
```

Notes:

- The template is a scaffold and expects secrets to be created in Key Vault via a secure pipeline or administrator action.
- Do not enable purge protection for Key Vault unless you understand the operational implications (it prevents permanent deletion).
