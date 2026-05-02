"""Common data transformations for prosperity indicators."""

from __future__ import annotations

from typing import TYPE_CHECKING

from pyspark.sql import functions as F

if TYPE_CHECKING:
    from pyspark.sql import DataFrame


def add_year_column(df: DataFrame, date_col: str = "date") -> DataFrame:
    """Extract a year integer column from a date string column.

    Args:
        df: Input DataFrame.
        date_col: Name of the date column.

    Returns:
        DataFrame with an additional ``year`` column.
    """
    return df.withColumn("year", F.year(F.col(date_col).cast("date")))


def compute_country_aggregates(df: DataFrame) -> DataFrame:
    """Compute average value per country and indicator.

    Args:
        df: Input DataFrame containing ``country_id``, ``indicator_id`` and ``value``.

    Returns:
        Aggregated DataFrame.
    """
    return df.groupBy("country_id", "indicator_id").agg(
        F.avg("value").alias("avg_value"),
        F.count("*").alias("record_count"),
        F.min("year").alias("min_year"),
        F.max("year").alias("max_year"),
    )
