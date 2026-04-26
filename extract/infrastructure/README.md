# Infrastructure Deployment (Azure)

This directory contains Terraform modules to deploy the Prosperity Lakehouse on **Azure**.

## Modules

| Module    | Purpose                                           |
|-----------|---------------------------------------------------|
| `datalake`| Creates the Resource Group, Storage Account, and Blob Container. |
| `compute` | Creates the Azure Function App with a Timer Trigger that runs the Rust extraction code on a schedule. |

## Prerequisites

- [Terraform](https://developer.hashicorp.com/terraform/downloads) ≥ 1.5
- [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli) (authenticated via `az login`)
- [Rust toolchain](https://rustup.rs/) with the `x86_64-unknown-linux-musl` target

## Deployment Steps

### 1. Datalake (Storage)

```bash
cd extract/infrastructure/datalake

# Copy and edit the example variables
cp terraform.tfvars.example terraform.tfvars
# terraform.tfvars
#   resource_group_name  = "rg-prosperity-lakehouse"
#   location             = "westeurope"
#   storage_account_name = "<globally-unique-name>"
#   container_name       = "transactions"

terraform init
terraform plan
terraform apply
```

Take note of the outputs, especially `storage_account_name` and `container_name` — you will need them for the compute module.

### 2. Compute (Azure Function)

```bash
cd extract/infrastructure/compute

# Copy and edit the example variables
cp terraform.tfvars.example terraform.tfvars
# terraform.tfvars
#   resource_group_name      = "rg-prosperity-lakehouse"
#   location                 = "westeurope"
#   function_app_name        = "func-prosperity-extract"
#   storage_account_name     = "<same-as-datalake>"
#   lakehouse_container_name = "transactions"
#   timer_schedule           = "0 0 2 * * *"   # daily at 02:00 UTC

terraform init
terraform plan
terraform apply
```

The compute module will:
1. Build the Rust binary (`extract/build-azure.sh`).
2. Package it with `host.json` and a Timer-trigger definition.
3. Deploy the ZIP to the Azure Function App via `az functionapp deployment source config-zip`.

## How It Works

- **Timer Trigger**: Azure Functions invokes the custom handler via HTTP POST to `/api/ProsperityExtract` on the CRON schedule.
- **Custom Handler**: The Rust binary (`handler`) starts an HTTP server on `FUNCTIONS_CUSTOMHANDLER_PORT`, receives the timer payload, and runs the same extraction + Delta-write logic used by the CLI.
- **Storage**: Data is written to the Azure Blob Container in Delta Lake format, partitioned by `indicator_id/country_id/year`.

## Useful Commands

```bash
# View function logs
az functionapp log tail --name <function-app-name> --resource-group <rg-name>

# Manually trigger the function (for testing)
curl -X POST https://<function-app>.azurewebsites.net/api/ProsperityExtract

# Destroy infrastructure
terraform destroy
```
