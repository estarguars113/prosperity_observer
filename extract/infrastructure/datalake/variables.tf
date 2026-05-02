variable "resource_group_name" {
  description = "Name of the Azure Resource Group"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "westeurope"
}

variable "storage_account_name" {
  description = "Globally unique name for the Azure Storage Account"
  type        = string
}

variable "container_name" {
  description = "Name of the Blob Container for the lakehouse"
  type        = string
  default     = "lakehouse"
}
