output "function_app_name" {
  description = "Name of the deployed Azure Function App"
  value       = azurerm_linux_function_app.lakehouse.name
}

output "function_app_default_hostname" {
  description = "Default hostname of the Function App"
  value       = azurerm_linux_function_app.lakehouse.default_hostname
}

output "application_insights_name" {
  description = "Name of the Application Insights resource"
  value       = azurerm_application_insights.lakehouse.name
}

output "timer_schedule" {
  description = "CRON schedule used for the timer trigger"
  value       = var.timer_schedule
}
