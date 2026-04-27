"""FastAPI application factory for galactic-api.

Startup sequence (inside the ASGI lifespan):
1. Build the async SQLAlchemy engine.
2. Connect to Redis and create the ``stream:propagate`` consumer group.
3. Run the initial TLE ingest (uses fallback if CelesTrak is unreachable).
4. Start the APScheduler 6-hour TLE refresh job.

Failures in steps 2–4 are logged but do not abort startup — the API
starts in a degraded state and the ``/v1/healthz`` endpoint reflects the
actual dependency health.
"""

from __future__ import annotations

import logging
import os
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager

import redis.asyncio as aioredis
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from fastapi import FastAPI
from sqlalchemy.ext.asyncio import AsyncEngine

from galactic_api.db.session import build_engine
from galactic_api.routers import healthz, satellites
from galactic_api.services.scheduler import build_scheduler
from galactic_api.services.tle_ingest import run_tle_ingest

logger = logging.getLogger(__name__)

# Name of the Redis stream and consumer group used by the propagation worker.
_STREAM_KEY = "stream:propagate"
_CONSUMER_GROUP = "workers"


async def _setup_redis(redis_url: str) -> aioredis.Redis:
    """Create an async Redis client and ensure the propagation consumer group exists."""
    client: aioredis.Redis = aioredis.from_url(redis_url, decode_responses=True)

    try:
        # MKSTREAM creates the stream if it does not yet exist.
        # BUSYGROUP is raised when the group already exists — that is fine.
        await client.xgroup_create(_STREAM_KEY, _CONSUMER_GROUP, id="$", mkstream=True)
        logger.info("Created Redis consumer group '%s' on '%s'", _CONSUMER_GROUP, _STREAM_KEY)
    except aioredis.ResponseError as exc:
        if "BUSYGROUP" in str(exc):
            logger.debug("Consumer group '%s' already exists — skipping creation", _CONSUMER_GROUP)
        else:
            logger.warning("Could not create Redis consumer group: %s", exc)

    return client


@asynccontextmanager
async def _lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """ASGI lifespan context manager — startup then teardown."""
    # ── Startup ──────────────────────────────────────────────────────────────
    engine: AsyncEngine = build_engine()
    app.state.engine = engine

    redis_url = os.environ.get("REDIS_URL", "redis://localhost:6379/0")
    try:
        redis_client = await _setup_redis(redis_url)
    except Exception:
        logger.warning("Redis setup failed at startup", exc_info=True)
        redis_client = aioredis.from_url(redis_url, decode_responses=True)
    app.state.redis = redis_client

    try:
        await run_tle_ingest(engine)
    except Exception:
        logger.warning("Initial TLE ingest failed at startup", exc_info=True)

    scheduler: AsyncIOScheduler = build_scheduler(engine)
    scheduler.start()
    app.state.scheduler = scheduler

    yield

    # ── Teardown ─────────────────────────────────────────────────────────────
    scheduler.shutdown(wait=False)
    await redis_client.aclose()
    await engine.dispose()


def create_app() -> FastAPI:
    """Create and configure the FastAPI application instance."""
    app = FastAPI(
        title="Galactic Guide API",
        version="0.1.0",
        description="Satellite tracking API — propagation-as-a-service.",
        lifespan=_lifespan,
    )
    app.include_router(healthz.router, prefix="/v1")
    app.include_router(satellites.router, prefix="/v1")
    return app


# Module-level singleton for ``uvicorn galactic_api.main:app``.
app = create_app()
