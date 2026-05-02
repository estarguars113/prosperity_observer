"""Delta Lake writer utilities."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pyspark.sql import DataFrame


def write_delta_table(
    df: DataFrame,
    table_path: str,
    *,
    mode: str = "overwrite",
    partition_by: list[str] | None = None,
    merge_schema: bool = False,
) -> None:
    """Write a DataFrame to a Delta table.

    Args:
        df: DataFrame to write.
        table_path: Destination path or URI.
        mode: Spark write mode (overwrite, append, etc.).
        partition_by: Columns to partition by.
        merge_schema: Allow schema evolution on overwrite.
    """
    writer = df.write.format("delta").mode(mode)

    if partition_by:
        writer = writer.partitionBy(*partition_by)
    if merge_schema:
        writer = writer.option("mergeSchema", "true")

    writer.save(table_path)
