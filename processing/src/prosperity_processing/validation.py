"""Data validation using Great Expectations."""

from __future__ import annotations

from typing import TYPE_CHECKING

import great_expectations as gx
import structlog

if TYPE_CHECKING:
    from pyspark.sql import DataFrame

logger = structlog.get_logger()


def validate_indicator_df(
    df: DataFrame,
    expectation_suite_name: str = "indicator_suite",
) -> dict:
    """Run Great Expectations validation on an indicator DataFrame.

    Args:
        df: DataFrame to validate.
        expectation_suite_name: Name of the expectation suite.

    Returns:
        Dictionary with validation results summary.
    """
    context = gx.get_context()

    data_source = context.data_sources.add_spark(name="spark_datasource")
    data_asset = data_source.add_dataframe_asset(name="indicator_asset")
    batch_definition = data_asset.add_batch_definition_whole_dataframe(
        "batch_def"
    )
    batch = batch_definition.get_batch(batch_parameters={"dataframe": df})

    try:
        suite = context.suites.get(name=expectation_suite_name)
    except gx.exceptions.ResourceNotFoundError:
        suite = gx.ExpectationSuite(name=expectation_suite_name)
        suite = context.suites.add(suite)

    # Add default expectations if suite is empty
    if not suite.expectations:
        suite.add_expectation(
            gx.expectations.ExpectColumnValuesToNotBeNull(column="indicator_id")
        )
        suite.add_expectation(
            gx.expectations.ExpectColumnValuesToNotBeNull(column="country_id")
        )
        suite.add_expectation(
            gx.expectations.ExpectColumnValuesToNotBeNull(column="year")
        )
        suite.add_expectation(
            gx.expectations.ExpectColumnValuesToBeBetween(
                column="value", min_value=0, max_value=100, mostly=0.95
            )
        )
        suite = context.suites.add_or_update(suite)

    validation_result = batch.validate(suite)

    success = validation_result.success
    logger.info(
        "validation_complete",
        suite=expectation_suite_name,
        success=success,
        evaluated_expectations=validation_result.statistics[
            "evaluated_expectations"
        ],
        successful_expectations=validation_result.statistics[
            "successful_expectations"
        ],
    )

    return {
        "success": success,
        "statistics": validation_result.statistics,
        "results": [
            {
                "expectation": r.expectation_config.type,
                "success": r.success,
                "unexpected_count": r.result.get("unexpected_count", 0),
            }
            for r in validation_result.results
        ],
    }
