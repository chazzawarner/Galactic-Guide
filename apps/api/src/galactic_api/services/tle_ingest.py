"""CelesTrak TLE ingest service.

Fetches Two-Line Element sets for the five curated satellites from CelesTrak
and upserts them into the ``tles`` table.  Falls back to the committed JSON
snapshot when CelesTrak is unreachable or ``OFFLINE=1`` is set.
"""

from __future__ import annotations

import json
import logging
import os
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import TypedDict

import httpx
from sqlalchemy.ext.asyncio import AsyncEngine, AsyncSession

from galactic_api.db.queries import upsert_tle

logger = logging.getLogger(__name__)

# CelesTrak GP data endpoint — returns a 3-line TLE block per satellite.
_CELESTRAK_URL = "https://celestrak.org/SATCAT/gp.php?CATNR={norad_id}&FORMAT=TLE"

# NORAD IDs for the five curated satellites (spec.md § Curated satellite dropdown).
CURATED_NORAD_IDS: list[int] = [25544, 20580, 44713, 36585, 33591]

_FALLBACK_PATH = Path(__file__).parent.parent.parent.parent / "data" / "celestrak-fallback.json"


class FallbackEntry(TypedDict):
    """Shape of one entry in celestrak-fallback.json."""

    norad_id: int
    name: str
    line1: str
    line2: str


def _parse_tle_epoch(line1: str) -> datetime:
    """Parse the epoch field from TLE Line 1 into an aware UTC datetime.

    The epoch is at characters 18–31 (0-indexed) in the format ``YYDDD.DDDDDDDD``,
    where ``YY`` is the 2-digit year and ``DDD.DDDDDDDD`` is the fractional day
    of year.
    """
    epoch_str = line1[18:32]
    year_2d = int(epoch_str[:2])
    year = 2000 + year_2d if year_2d < 57 else 1900 + year_2d  # noqa: PLR2004
    day_frac = float(epoch_str[2:])
    day = int(day_frac)
    frac = day_frac - day
    return datetime(year, 1, 1, tzinfo=UTC) + timedelta(days=day - 1, seconds=frac * 86400)


def _load_fallback() -> list[FallbackEntry]:
    """Load the committed TLE snapshot from disk."""
    with _FALLBACK_PATH.open() as fh:
        data: list[FallbackEntry] = json.load(fh)
    return data


async def _ingest_entries(
    session: AsyncSession,
    entries: list[FallbackEntry],
) -> None:
    """Upsert a list of TLE entries into the database.

    Commits once after all entries are processed.  If an individual upsert
    fails the session is rolled back so subsequent entries can proceed in a
    clean state; the failed entry is logged and skipped.
    """
    for entry in entries:
        try:
            epoch = _parse_tle_epoch(entry["line1"])
            await upsert_tle(
                session,
                norad_id=entry["norad_id"],
                line1=entry["line1"],
                line2=entry["line2"],
                epoch=epoch,
            )
        except Exception:
            logger.exception("Failed to upsert TLE for NORAD %d", entry["norad_id"])
            try:
                await session.rollback()
            except Exception:
                logger.exception(
                    "Failed to roll back session after upsert failure for NORAD %d",
                    entry["norad_id"],
                )
    await session.commit()


async def _fetch_from_celestrak(norad_id: int, client: httpx.AsyncClient) -> FallbackEntry | None:
    """Fetch a single satellite TLE from CelesTrak.

    Returns ``None`` on any HTTP or parse error.
    """
    url = _CELESTRAK_URL.format(norad_id=norad_id)
    try:
        response = await client.get(url, timeout=10.0)
        response.raise_for_status()
    except Exception:
        logger.warning("CelesTrak request failed for NORAD %d", norad_id)
        return None

    lines = [ln.strip() for ln in response.text.splitlines() if ln.strip()]
    if len(lines) < 3:  # noqa: PLR2004
        logger.warning(
            "Unexpected CelesTrak response for NORAD %d: %r", norad_id, response.text[:100]
        )
        return None

    name, line1, line2 = lines[0], lines[1], lines[2]
    return FallbackEntry(norad_id=norad_id, name=name, line1=line1, line2=line2)


async def run_tle_ingest(engine: AsyncEngine) -> None:
    """Main ingest entry point called at startup and every 6 hours.

    Uses the offline fallback when:
    - ``OFFLINE=1`` is set in the environment, **or**
    - CelesTrak returns an error for a particular satellite (per-satellite fallback), **or**
    - CelesTrak returns an error for all satellites (full fallback).

    Failures are logged but never raised so that a transient CelesTrak outage
    does not prevent the API from starting.
    """
    offline = os.environ.get("OFFLINE", "0") == "1"

    if offline:
        logger.info("OFFLINE=1 — loading TLEs from fallback snapshot")
        entries: list[FallbackEntry] = _load_fallback()
    else:
        fallback_by_norad = {e["norad_id"]: e for e in _load_fallback()}
        entries = []
        async with httpx.AsyncClient() as client:
            for norad_id in CURATED_NORAD_IDS:
                entry = await _fetch_from_celestrak(norad_id, client)
                if entry is not None:
                    entries.append(entry)
                else:
                    # Fall back to the committed snapshot for this satellite so
                    # we always have at least one TLE row per satellite.
                    fb = fallback_by_norad.get(norad_id)
                    if fb is not None:
                        logger.info(
                            "Using fallback TLE for NORAD %d (CelesTrak unavailable)", norad_id
                        )
                        entries.append(fb)

    async with AsyncSession(engine) as session:
        await _ingest_entries(session, entries)

    logger.info("TLE ingest complete (%d entries processed)", len(entries))
