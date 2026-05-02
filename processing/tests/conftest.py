"""Shared test fixtures."""

from __future__ import annotations

import pytest

from prosperity_processing.pyspark_session import build_spark_session


@pytest.fixture(scope="session")
def spark():
    """Provide a SparkSession for the test suite."""
    session = build_spark_session(app_name="prosperity-test")
    yield session
    session.stop()
