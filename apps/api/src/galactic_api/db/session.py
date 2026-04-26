"""Async SQLAlchemy engine and session factory."""

from __future__ import annotations

import os
from collections.abc import AsyncGenerator

from sqlalchemy.ext.asyncio import (
    AsyncEngine,
    AsyncSession,
    async_sessionmaker,
    create_async_engine,
)


def build_engine() -> AsyncEngine:
    """Create an async SQLAlchemy engine from the DATABASE_URL environment variable."""
    url = os.environ["DATABASE_URL"]
    return create_async_engine(url, pool_pre_ping=True)


def build_session_factory(engine: AsyncEngine) -> async_sessionmaker[AsyncSession]:
    """Return a session factory bound to *engine*."""
    return async_sessionmaker(engine, expire_on_commit=False)


async def get_session(
    session_factory: async_sessionmaker[AsyncSession],
) -> AsyncGenerator[AsyncSession, None]:
    """FastAPI dependency: yield a database session then close it."""
    async with session_factory() as session:
        yield session


async def check_postgres(engine: AsyncEngine) -> None:
    """Execute ``SELECT 1`` to verify that Postgres is reachable.

    Raises on any connection or query failure.
    """
    async with engine.connect() as conn:
        await conn.execute(__import__("sqlalchemy").text("SELECT 1"))
