"""PySpark session builder with Delta Lake support."""

from __future__ import annotations

import os

from delta import configure_spark_with_delta_pip
from pyspark.sql import SparkSession

DEFAULT_DELTA_JARS = [
    "io.delta:delta-core_2.12:2.4.0",
    "io.delta:delta-storage:2.4.0",
]


def build_spark_session(
    app_name: str = "prosperity-processing",
    *,
    local_mode: bool = True,
    extra_packages: list[str] | None = None,
) -> SparkSession:
    """Build a SparkSession with Delta Lake extensions.

    Args:
        app_name: Name of the Spark application.
        local_mode: Whether to run locally with all cores.
        extra_packages: Additional Maven packages to include.

    Returns:
        Configured SparkSession.
    """
    builder = (
        SparkSession.builder.appName(app_name)
        .config("spark.sql.extensions", "io.delta.sql.DeltaSparkSessionExtension")
        .config(
            "spark.sql.catalog.spark_catalog",
            "org.apache.spark.sql.delta.catalog.DeltaCatalog",
        )
    )

    if local_mode:
        builder = builder.master("local[*]")

    packages = list(extra_packages or [])
    if packages:
        builder = builder.config("spark.jars.packages", ",".join(packages))

    spark = configure_spark_with_delta_pip(builder).getOrCreate()
    spark.sparkContext.setLogLevel(
        os.environ.get("SPARK_LOG_LEVEL", "WARN")
    )
    return spark
