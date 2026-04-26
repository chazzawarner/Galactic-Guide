"""GET /v1/healthz — liveness and readiness probe.

Returns 200 when both Postgres and Redis are reachable; returns 503 when
either dependency is down.  The response shape is defined in
``galactic_api.models.health``.
"""

from __future__ import annotations

import asyncio
import logging
from datetime import UTC, datetime

from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse

from galactic_api.db.session import check_postgres
from galactic_api.models.health import Checks, HealthResponse

logger = logging.getLogger(__name__)

router = APIRouter()

_VERSION = "0.1.0"


async def _check_redis_health(request: Request) -> str:
    """Ping Redis; return ``"ok"`` or ``"fail"``."""
    try:
        await request.app.state.redis.ping()
        return "ok"
    except Exception:
        logger.warning("Redis health check failed", exc_info=True)
        return "fail"


async def _check_postgres_health(request: Request) -> str:
    """Run ``SELECT 1`` against Postgres; return ``"ok"`` or ``"fail"``."""
    try:
        await check_postgres(request.app.state.engine)
        return "ok"
    except Exception:
        logger.warning("Postgres health check failed", exc_info=True)
        return "fail"


@router.get(
    "/healthz",
    response_model=HealthResponse,
    summary="Liveness and readiness probe",
    tags=["ops"],
)
async def healthz(request: Request) -> JSONResponse:
    """Return 200 when both dependencies are healthy; 503 when degraded.

    Both checks run concurrently via :func:`asyncio.gather`.
    No caching headers are set on this endpoint.
    """
    pg_status, redis_status = await asyncio.gather(
        _check_postgres_health(request),
        _check_redis_health(request),
    )

    overall = "ok" if pg_status == "ok" and redis_status == "ok" else "degraded"
    payload = HealthResponse(
        status=overall,
        checks=Checks(postgres=pg_status, redis=redis_status),
        version=_VERSION,
        now=datetime.now(tz=UTC).isoformat(),
    )
    status_code = 200 if overall == "ok" else 503
    return JSONResponse(content=payload.model_dump(), status_code=status_code)
