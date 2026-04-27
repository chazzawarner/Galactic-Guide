"""APScheduler wrapper that runs the TLE ingest job every 6 hours."""

from __future__ import annotations

import logging

from apscheduler.schedulers.asyncio import AsyncIOScheduler
from sqlalchemy.ext.asyncio import AsyncEngine

from galactic_api.services.tle_ingest import run_tle_ingest

logger = logging.getLogger(__name__)

_SIX_HOURS = 6 * 3600


def build_scheduler(engine: AsyncEngine) -> AsyncIOScheduler:
    """Create and configure an :class:`AsyncIOScheduler` for TLE refresh.

    The scheduler is *not* started here; call :meth:`~AsyncIOScheduler.start`
    after the application is fully initialised.
    """
    scheduler: AsyncIOScheduler = AsyncIOScheduler()
    scheduler.add_job(
        run_tle_ingest,
        "interval",
        seconds=_SIX_HOURS,
        id="tle-refresh",
        args=[engine],
        max_instances=1,
        replace_existing=True,
        misfire_grace_time=300,
    )
    return scheduler
