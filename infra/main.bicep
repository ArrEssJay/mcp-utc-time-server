// Infra scaffold for deploying Container App + Key Vault + Log Analytics + Managed Identity
// This file is a starting point. It intentionally avoids embedding secret values and instead
// expects secrets to be created in Key Vault (or via deployment pipelines) and referenced
// by the Container App at runtime using a user-assigned managed identity.

@description('Name of the Container App environment')
param environmentName string

@description('Location for all resources')
param location string = resourceGroup().location

@description('Container App name')
param containerAppName string

@description('Container Registry name (existing or new)')
param acrName string

@description('Key Vault name')
param keyVaultName string

@description('Log Analytics workspace name')
param logAnalyticsName string

// User-assigned managed identity for the Container App to access Key Vault and ACR
resource userIdentity 'Microsoft.ManagedIdentity/userAssignedIdentities@2018-11-30' = {
  name: '${containerAppName}-identity'
  location: location
}

// Container Registry - created here as an option; set acrName to an existing registry name
resource acr 'Microsoft.ContainerRegistry/registries@2021-09-01' = if (!empty(acrName)) {
  name: acrName
  location: location
  sku: {
    name: 'Standard'
  }
  properties: {
    adminUserEnabled: false
  }
}

// Key Vault - DO NOT disable purgeProtection in production. Soft-delete and purge protection
// help prevent accidental or malicious deletion of secrets. Secrets themselves should be
// created via a secure CI/CD step or by administrators after Key Vault creation.
resource keyVault 'Microsoft.KeyVault/vaults@2021-06-01-preview' = {
  name: keyVaultName
  location: location
  properties: {
    tenantId: subscription().tenantId
    sku: {
      name: 'standard'
      family: 'A'
    }
    accessPolicies: [] // prefer Azure RBAC for Key Vault; populate via scripts if needed
    enableSoftDelete: true
    enablePurgeProtection: true
    // networkAcls can be configured here to restrict access to trusted networks
  }
}

// Log Analytics workspace for Container Apps diagnostics (required for Container Apps)
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2021-06-01' = {
  name: logAnalyticsName
  location: location
  properties: {
    sku: {
      name: 'PerGB2018'
    }
  }
}

// Container Apps Managed Environment - this example demonstrates wiring to Log Analytics
// For production use, pick API versions and properties consistent with current Azure docs
resource containerEnv 'Microsoft.Web/managedEnvironments@2022-03-01' = {
  name: environmentName
  location: location
  properties: {
    appLogsConfiguration: {
      destination: 'log-analytics'
      logAnalyticsConfiguration: {
        customerId: logAnalytics.properties.customerId
        sharedKey: listKeys(logAnalytics.id, '2021-06-01').primarySharedKey
      }
    }
  }
}

// Assign AcrPull role to the user-assigned managed identity for the registry
// RoleDefinitionId for AcrPull: 7f951dda-4ed3-4680-a7ca-43fe172d538d
resource acrPullRole 'Microsoft.Authorization/roleAssignments@2020-04-01-preview' = if (!empty(acrName)) {
  name: guid(acr.id, userIdentity.properties.principalId, 'acrpull')
  scope: acr
  properties: {
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', '7f951dda-4ed3-4680-a7ca-43fe172d538d')
    principalId: userIdentity.properties.principalId
    principalType: 'ServicePrincipal'
  }
}

// NOTE: We do not create Container App resources here with secrets. Instead:
//  - Create secrets in Key Vault (API keys, NTP server lists, etc.) via secure pipeline or admin
//  - Grant the user-assigned managed identity access to Key Vault (Key Vault RBAC or access policies)
//  - Configure the Container App to use the managed identity and pull secrets from Key Vault

// Output values useful for CI/CD
output RESOURCE_GROUP_ID string = resourceGroup().id
output AZURE_CONTAINER_REGISTRY_ENDPOINT string = if (empty(acrName)) then '' else '${acrName}.azurecr.io'
output MANAGED_IDENTITY_PRINCIPAL_ID string = userIdentity.properties.principalId
