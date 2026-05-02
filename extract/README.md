# Prosperity Lakehouse

Fetch World Bank prosperity indicators and store them in a **Delta Lake** table partitioned by `indicator_id / country_id / year`.

Supported storage backends:
- **Local** filesystem
- **AWS S3**
- **Azure Blob Storage**

---

## Usage

The CLI has three subcommands — one for each storage backend.  All of them write to the **same partitioned Delta table**; only the root location changes.

```bash
# Local mode (Delta table on local disk)
cargo run -- local

# AWS S3 mode (Delta table on S3)
cargo run -- aws

# Azure Blob Storage mode (Delta table on Azure)
cargo run -- azure
```

---

## Partitioning

Data is physically partitioned by three columns:

```
indicator_id / country_id / year
```

Example on-disk layout (local or cloud):

```
transactions/
  indicator_id=SP.POP.TOTL/
    country_id=AFE/
      year=2024/
        part-00001.parquet
      year=2023/
        part-00001.parquet
  indicator_id=SI.POV.DDAY/
    country_id=USA/
      year=2024/
        part-00001.parquet
  _delta_log/
    00000000000000000000.json
```

The `_delta_log` directory contains the Delta transaction log.  The actual data lives in Parquet files under the Hive-style partition directories.

---

## Environment Variables

### General

| Variable | Default | Description |
|----------|---------|-------------|
| `LAKEHOUSE_LOCAL_PATH` | `./lakehouse` | Base directory for **local** storage |
| `LAKEHOUSE_TABLE_NAME` | `transactions` | Name of the Delta table (used as the final path segment) |

### AWS S3

| Variable | Required? | Description |
|----------|-----------|-------------|
| `AWS_S3_BUCKET` | **Yes** | S3 bucket name |
| `AWS_ACCESS_KEY_ID` | Yes* | AWS access key |
| `AWS_SECRET_ACCESS_KEY` | Yes* | AWS secret key |
| `AWS_DEFAULT_REGION` | Yes* | AWS region (e.g. `us-east-1`) |
| `AWS_ENDPOINT_URL` | No | Custom S3 endpoint (e.g. for MinIO) |

> *These standard AWS credentials are read automatically by the underlying `object_store` library.  You may also rely on IAM roles / instance profiles when running inside AWS infrastructure.

### Azure Blob Storage

| Variable | Required? | Description |
|----------|-----------|-------------|
| `AZURE_CONTAINER_NAME` | **Yes** | Blob container name |
| `AZURE_STORAGE_ACCOUNT_NAME` | **Yes** | Storage account name |
| `AZURE_STORAGE_ACCOUNT_KEY` | **Yes** | Storage account access key |

> The Azure backend uses `az://{container}/{table_name}` URIs internally.  Credentials are picked up from the environment by the `object_store` Azure client.

---

## Examples

### Local run

```bash
cargo run -- local
# Table will be created at ./lakehouse/transactions
```

### AWS S3 run

```bash
export AWS_S3_BUCKET=my-bucket
export AWS_ACCESS_KEY_ID=AKIA...
export AWS_SECRET_ACCESS_KEY=secret...
export AWS_DEFAULT_REGION=us-east-1

cargo run -- aws
# Table will be created at s3://my-bucket/transactions
```

### Azure Blob Storage run

```bash
export AZURE_CONTAINER_NAME=my-container
export AZURE_STORAGE_ACCOUNT_NAME=myaccount
export AZURE_STORAGE_ACCOUNT_KEY=base64-key...

cargo run -- azure
# Table will be created at az://my-container/transactions
```

or for local testing still, but with azure configuration

```bash
docker run -p 10000:10000 -p 10001:10001 -p 10002:10002 mcr.microsoft.com/azure-storage/azurite
```

---

## Indicator Download Status

The pipeline fetches **6 World Bank indicators** concurrently. Each indicator is retried up to **2 times** on failure before being skipped.

### Configured Indicators

| # | ID | Name | Category | Status |
|---|-----|------|----------|--------|
| 1 | `SP.DYN.LE00.IN` | Life Expectancy | Health | ✅ Active |
| 2 | `SH.H2O.SMDW.ZS` | Water Access | Water Sanitation | ✅ Active |
| 3 | `SL.TLF.ADVN.ZS` | Education Access | Education Access | ✅ Active |
| 4 | `SI.POV.DDAY` | Poverty Rate | Poverty | ✅ Active |
| 5 | `SL.UEM.TOTL.ZS` | Unemployment Rate | Employment | ✅ Active |
| 6 | `ER.PTD.TOTL.ZS` | Protected Areas | Conservation | ✅ Active |

> **Last verified:** 2026-04-25 — all 6 indicators verified against the World Bank API.


### What Happens When an Indicator Fails?

The pipeline uses **graceful failure handling**:

1. **Retry policy:** Each indicator is retried 2 times with a 1-second delay.
2. **Failure detection:** If an indicator still fails after retries, it is recorded as a failure.
3. **Atomic behavior:** If **any** indicator fails, **no data is written** to the Delta table and the program exits with a **non-zero status code**.
4. **Failure summary:** A table of all failed indicators is printed to `stderr` before exit.

**Example output when failures occur:**
```
❌ FAILED INDICATORS:
ID                             Name                      Error
------------------------------------------------------------------------------------------------------------------------
UHC_SCI                        Health Access             World Bank API error: [{"message":[{"id":"120",...
WB_HCP_EMP_NIFL_A              Informal Employment       World Bank API error: [{"message":[{"id":"120",...

Error: 2 of 6 indicators failed to fetch — no data written to lakehouse
```

### Known Failure Reasons

| Reason | Description |
|--------|-------------|
| **HTTP 404** | Indicator code does not exist in the World Bank API |
| **Rate limiting** | Too many concurrent requests; wait and retry |
| **Timeout** | Request exceeded 30 seconds |
| **Empty response** | API returned valid JSON but no data array |

---

## Schema

The Delta table has the following schema:

| Column | Type | Nullable | Partition? |
|--------|------|----------|------------|
| `indicator_id` | String | No | **Yes** |
| `indicator_name` | String | No | No |
| `country_id` | String | No | **Yes** |
| `country_name` | String | No | No |
| `year` | String | No | **Yes** |
| `value` | Double | Yes | No |
