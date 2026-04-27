"""Pytest fixtures for galactic_api tests.

Uses testcontainers to spin up real Postgres and Redis instances so the
tests do not depend on external services.
"""

from __future__ import annotations

from collections.abc import Generator

import pytest
from testcontainers.postgres import PostgresContainer
from testcontainers.redis import RedisContainer


@pytest.fixture(scope="session")
def pg_container() -> Generator[PostgresContainer, None, None]:
    """Start a Postgres 16 container for the entire test session."""
    with PostgresContainer("postgres:16", driver="asyncpg") as postgres:
        yield postgres


@pytest.fixture(scope="session")
def redis_container() -> Generator[RedisContainer, None, None]:
    """Start a Redis 7 container for the entire test session."""
    with RedisContainer("redis:7") as rc:
        yield rc
