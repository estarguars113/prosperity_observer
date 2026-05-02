"""Delta Lake reader utilities."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pyspark.sql import DataFrame, SparkSession


def read_delta_table(spark: SparkSession, table_path: str) -> DataFrame:
    """Read a Delta table into a Spark DataFrame.

    Args:
        spark: Active SparkSession.
        table_path: Path or URI to the Delta table.

    Returns:
        Spark DataFrame.
    """
    return spark.read.format("delta").load(table_path)


def read_delta_version(
    spark: SparkSession, table_path: str, version: int
) -> DataFrame:
    """Read a specific version of a Delta table.

    Args:
        spark: Active SparkSession.
        table_path: Path or URI to the Delta table.
        version: Table version to read.

    Returns:
        Spark DataFrame at the requested version.
    """
    return (
        spark.read.format("delta")
        .option("versionAsOf", version)
        .load(table_path)
    )
