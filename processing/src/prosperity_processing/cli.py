"""CLI entrypoint for local processing and validation."""

from __future__ import annotations

import structlog
import typer

from prosperity_processing.pyspark_session import build_spark_session
from prosperity_processing.reader import read_delta_table
from prosperity_processing.transforms import compute_country_aggregates
from prosperity_processing.validation import validate_indicator_df
from prosperity_processing.writer import write_delta_table

app = typer.Typer(help="Prosperity Lakehouse processing CLI")
logger = structlog.get_logger()


@app.command()
def process(
    input_path: str = typer.Argument(..., help="Path to raw Delta table"),
    output_path: str = typer.Argument(..., help="Path for processed Delta table"),
) -> None:
    """Read raw indicators, validate, transform and write processed table."""
    spark = build_spark_session()
    df = read_delta_table(spark, input_path)

    logger.info("data_loaded", rows=df.count(), columns=len(df.columns))

    validation = validate_indicator_df(df)
    if not validation["success"]:
        logger.warning("validation_failed", details=validation["statistics"])
        raise typer.Exit(code=1)

    transformed = compute_country_aggregates(df)
    write_delta_table(
        transformed,
        output_path,
        mode="overwrite",
        partition_by=["indicator_id"],
    )
    logger.info("processing_complete", output_path=output_path)


@app.command()
def validate(
    input_path: str = typer.Argument(..., help="Path to Delta table to validate"),
) -> None:
    """Run Great Expectations validation on a Delta table."""
    spark = build_spark_session()
    df = read_delta_table(spark, input_path)
    result = validate_indicator_df(df)
    typer.echo(result)


if __name__ == "__main__":
    app()
