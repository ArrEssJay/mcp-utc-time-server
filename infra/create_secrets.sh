#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <key-vault-name> [key-names...]"
  echo "Example: $0 my-mcp-kv api-key-1 api-key-2 api-key-admin"
  exit 2
fi

VAULT_NAME=$1
shift

if ! command -v az >/dev/null 2>&1; then
  echo "az CLI not found. Please install and login (az login)."
  exit 2
fi

for KEY_NAME in "$@"; do
  read -s -p "Enter value for secret $KEY_NAME: " SECRET_VAL
  echo
  if [ -z "$SECRET_VAL" ]; then
    echo "Empty secret, skipping $KEY_NAME"
    continue
  fi
  echo "Setting secret $KEY_NAME in Key Vault $VAULT_NAME..."
  az keyvault secret set --vault-name "$VAULT_NAME" --name "$KEY_NAME" --value "$SECRET_VAL" >/dev/null
  echo "Secret $KEY_NAME set."
done

echo "All done. Use Azure RBAC or Key Vault policies to grant access to the managed identity."
