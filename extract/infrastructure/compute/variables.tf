variable "resource_group_name" {
  description = "Name of the existing Azure Resource Group"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "westeurope"
}

variable "function_app_name" {
  description = "Globally unique name for the Azure Function App"
  type        = string
}

variable "storage_account_name" {
  description = "Name of the existing Storage Account used for the lakehouse (also used by Function App)"
  type        = string
}

variable "lakehouse_container_name" {
  description = "Name of the Blob Container where Delta data is written"
  type        = string
  default     = "transactions"
}

variable "timer_schedule" {
  description = "CRON expression for the timer trigger (default: daily at 02:00 UTC)"
  type        = string
  default     = "0 0 2 * * *"
}

variable "rust_project_path" {
  description = "Path to the Rust project root relative to the compute module"
  type        = string
  default     = "../.."
}
