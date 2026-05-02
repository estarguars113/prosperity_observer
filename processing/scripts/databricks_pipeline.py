"""Databricks pipeline script for prosperity indicator processing.

This script is designed to run as a Databricks notebook or job.
It expects a SparkSession to be provided by the Databricks runtime.
"""

from __future__ import annotations

from pyspark.sql import SparkSession

from prosperity_processing.reader import read_delta_table
from prosperity_processing.transforms import compute_country_aggregates
from prosperity_processing.validation import validate_indicator_df
from prosperity_processing.writer import write_delta_table


def run_pipeline(
    input_path: str,
    output_path: str,
    expectation_suite: str = "indicator_suite",
) -> dict:
    """Execute the full processing pipeline on Databricks.

    Args:
        input_path: Path or URI to the raw Delta table.
        output_path: Path or URI for the processed Delta table.
        expectation_suite: Great Expectations suite name.

    Returns:
        Summary dictionary with validation and row counts.
    """
    spark = SparkSession.builder.getOrCreate()

    df = read_delta_table(spark, input_path)
    raw_count = df.count()

    validation = validate_indicator_df(df, expectation_suite)
    if not validation["success"]:
        raise ValueError(f"Validation failed: {validation['statistics']}")

    transformed = compute_country_aggregates(df)
    write_delta_table(
        transformed,
        output_path,
        mode="overwrite",
        partition_by=["indicator_id"],
    )

    return {
        "raw_count": raw_count,
        "aggregated_count": transformed.count(),
        "validation": validation,
    }


if __name__ == "__main__":
    # Example usage when running as a standalone script
    result = run_pipeline(
        input_path="dbfs:/mnt/prosperity/raw/transactions",
        output_path="dbfs:/mnt/prosperity/processed/aggregated",
    )
    print(result)
