# Prosperity Processing

Python package for processing, validating, and exploring prosperity indicator data in the lakehouse.

## Features

- **PySpark + Delta Lake**: Read and write partitioned Delta tables locally or on Databricks.
- **Data Validation**: Great Expectations integration for quality checks.
- **Exploration**: JupyterLab setup with PySpark kernel for interactive analysis.
- **Databricks Ready**: Scripts are designed to run unchanged on Databricks clusters.

## Quick Start

```bash
cd processing
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

### Run locally

```bash
# Process raw indicators into aggregated table
prosperity-process ./lakehouse/transactions ./lakehouse/aggregated

# Validate a table
prosperity-validate ./lakehouse/transactions
```

### Run on Databricks

Upload `scripts/databricks_pipeline.py` to a Databricks workspace and attach it to a cluster with Delta Lake enabled.

## Project Structure

```
processing/
├── pyproject.toml
├── src/prosperity_processing/
│   ├── __init__.py
│   ├── pyspark_session.py   # Spark session builder
│   ├── reader.py            # Delta read utilities
│   ├── writer.py            # Delta write utilities
│   ├── transforms.py        # Data transformations
│   ├── validation.py        # Great Expectations checks
│   └── cli.py               # Typer CLI
├── scripts/
│   └── databricks_pipeline.py
├── notebooks/
│   └── exploration.ipynb
└── tests/
```
