"""Unit tests for data transformations."""

from __future__ import annotations

from prosperity_processing.transforms import add_year_column, compute_country_aggregates


def test_add_year_column(spark):
    """Test year extraction from date column."""
    df = spark.createDataFrame(
        [("2020-01-15",), ("2021-06-30",)],
        ["date"],
    )
    result = add_year_column(df)
    assert "year" in result.columns
    years = [row.year for row in result.collect()]
    assert years == [2020, 2021]


def test_compute_country_aggregates(spark):
    """Test country aggregation logic."""
    df = spark.createDataFrame(
        [
            ("IND1", "CountryA", "2020", 10.0),
            ("IND1", "CountryA", "2021", 20.0),
            ("IND1", "CountryB", "2020", 5.0),
        ],
        ["indicator_id", "country_id", "year", "value"],
    )
    result = compute_country_aggregates(df)
    rows = {row.country_id: row for row in result.collect()}
    assert len(rows) == 2
    assert rows["CountryA"].avg_value == 15.0
    assert rows["CountryA"].record_count == 2
