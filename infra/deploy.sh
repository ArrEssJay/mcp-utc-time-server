#!/usr/bin/env bash
set -euo pipefail

RG=${1:-mcp-time-rg}
TEMPLATE_FILE="$(dirname "$0")/main.bicep"
PARAMS_FILE="$(dirname "$0")/main.parameters.json"

echo "Running az deployment group what-if for resource group: $RG"
az deployment group what-if --resource-group "$RG" --template-file "$TEMPLATE_FILE" --parameters "@$PARAMS_FILE"

echo "To deploy for real run:
az deployment group create --resource-group $RG --template-file $TEMPLATE_FILE --parameters @$PARAMS_FILE"
