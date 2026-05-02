output "resource_group_name" {
  description = "Name of the created Resource Group"
  value       = azurerm_resource_group.lakehouse.name
}

output "storage_account_name" {
  description = "Name of the created Storage Account"
  value       = azurerm_storage_account.lakehouse.name
}

output "storage_account_id" {
  description = "ID of the created Storage Account"
  value       = azurerm_storage_account.lakehouse.id
}

output "container_name" {
  description = "Name of the created Blob Container"
  value       = azurerm_storage_container.lakehouse.name
}

output "primary_blob_endpoint" {
  description = "Primary blob endpoint URL"
  value       = azurerm_storage_account.lakehouse.primary_blob_endpoint
}

output "primary_access_key" {
  description = "Primary access key for the Storage Account"
  value       = azurerm_storage_account.lakehouse.primary_access_key
  sensitive   = true
}
