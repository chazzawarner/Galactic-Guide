"""Alembic environment configuration — async SQLAlchemy + asyncpg.

The DATABASE_URL is read from the environment at runtime, so that the same
alembic.ini works in every environment without baking in credentials.
"""

from __future__ import annotations

import asyncio
import os
from logging.config import fileConfig

from sqlalchemy import pool
from sqlalchemy.engine import Connection
from sqlalchemy.ext.asyncio import async_engine_from_config

from alembic import context

# ---------------------------------------------------------------------------
# Alembic Config object (gives access to values in alembic.ini)
# ---------------------------------------------------------------------------
config = context.config

# Set up Python standard-library logging from the alembic.ini [loggers] section.
if config.config_file_name is not None:
    fileConfig(config.config_file_name)

# ---------------------------------------------------------------------------
# Inject DATABASE_URL from the environment, overriding the placeholder in
# alembic.ini.  Alembic's synchronous psycopg2 driver is not available here;
# we use asyncpg throughout and run migrations via run_sync().
# ---------------------------------------------------------------------------
database_url = os.environ["DATABASE_URL"]
config.set_main_option("sqlalchemy.url", database_url)


# ---------------------------------------------------------------------------
# Offline mode — emit raw SQL without a live connection
# ---------------------------------------------------------------------------
def run_migrations_offline() -> None:
    """Run migrations without a database connection (SQL output only)."""
    url = config.get_main_option("sqlalchemy.url")
    context.configure(
        url=url,
        literal_binds=True,
        dialect_opts={"paramstyle": "named"},
    )
    with context.begin_transaction():
        context.run_migrations()


# ---------------------------------------------------------------------------
# Online mode — async connection via asyncpg
# ---------------------------------------------------------------------------
def do_run_migrations(connection: Connection) -> None:
    """Execute pending migrations on the given synchronous connection handle."""
    context.configure(connection=connection)
    with context.begin_transaction():
        context.run_migrations()


async def run_async_migrations() -> None:
    """Open an async engine and bridge to the synchronous migration runner."""
    connectable = async_engine_from_config(
        config.get_section(config.config_ini_section, {}),
        prefix="sqlalchemy.",
        poolclass=pool.NullPool,
    )
    async with connectable.connect() as connection:
        await connection.run_sync(do_run_migrations)
    await connectable.dispose()


def run_migrations_online() -> None:
    """Entry point for online (connected) migrations."""
    asyncio.run(run_async_migrations())


# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------
if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
