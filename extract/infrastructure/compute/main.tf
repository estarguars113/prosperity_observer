terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.100"
    }
  }
}

provider "azurerm" {
  features {}
}

# ---------------------------------------------------------------------------
# Reference existing datalake resources
# ---------------------------------------------------------------------------
data "azurerm_resource_group" "lakehouse" {
  name = var.resource_group_name
}

data "azurerm_storage_account" "lakehouse" {
  name                = var.storage_account_name
  resource_group_name = data.azurerm_resource_group.lakehouse.name
}

# ---------------------------------------------------------------------------
# Service Plan (Consumption / serverless)
# ---------------------------------------------------------------------------
resource "azurerm_service_plan" "lakehouse" {
  name                = "${var.function_app_name}-plan"
  resource_group_name = data.azurerm_resource_group.lakehouse.name
  location            = var.location
  os_type             = "Linux"
  sku_name            = "Y1"
}

# ---------------------------------------------------------------------------
# Application Insights
# ---------------------------------------------------------------------------
resource "azurerm_application_insights" "lakehouse" {
  name                = "${var.function_app_name}-insights"
  resource_group_name = data.azurerm_resource_group.lakehouse.name
  location            = var.location
  application_type    = "other"
}

# ---------------------------------------------------------------------------
# Function App
# ---------------------------------------------------------------------------
resource "azurerm_linux_function_app" "lakehouse" {
  name                = var.function_app_name
  resource_group_name = data.azurerm_resource_group.lakehouse.name
  location            = var.location

  storage_account_name       = data.azurerm_storage_account.lakehouse.name
  storage_account_access_key = data.azurerm_storage_account.lakehouse.primary_access_key
  service_plan_id            = azurerm_service_plan.lakehouse.id

  site_config {
    application_stack {
      use_custom_runtime = true
    }
    application_insights_connection_string = azurerm_application_insights.lakehouse.connection_string
  }

  app_settings = {
    FUNCTIONS_WORKER_RUNTIME       = "custom"
    WEBSITE_RUN_FROM_PACKAGE       = "1"
    AZURE_STORAGE_ACCOUNT_NAME     = data.azurerm_storage_account.lakehouse.name
    AZURE_CONTAINER_NAME           = var.lakehouse_container_name
    AZURE_STORAGE_ACCOUNT_KEY      = data.azurerm_storage_account.lakehouse.primary_access_key
    LAKEHOUSE_TABLE_NAME           = "transactions"
    RUST_BACKTRACE                 = "1"
    Schedule                       = var.timer_schedule
  }
}

# ---------------------------------------------------------------------------
# Build & ZIP the Rust custom handler
# ---------------------------------------------------------------------------
resource "null_resource" "build_rust_handler" {
  triggers = {
    always_run = timestamp()
  }

  provisioner "local-exec" {
    working_dir = path.module
    command     = "bash ${var.rust_project_path}/build-azure.sh"
  }
}

# ---------------------------------------------------------------------------
# Deploy the ZIP package using az cli
# ---------------------------------------------------------------------------
resource "null_resource" "zip_deploy" {
  depends_on = [null_resource.build_rust_handler]

  triggers = {
    build_id = null_resource.build_rust_handler.id
  }

  provisioner "local-exec" {
    command = <<EOT
      az functionapp deployment source config-zip \
        --resource-group ${data.azurerm_resource_group.lakehouse.name} \
        --name ${azurerm_linux_function_app.lakehouse.name} \
        --src ${var.rust_project_path}/target/azure/function.zip \
        --build-remote false
    EOT
  }
}
