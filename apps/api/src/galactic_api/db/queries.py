"""Typed query helpers for the galactic_api data layer."""

from __future__ import annotations

from datetime import datetime

from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession


async def upsert_tle(
    session: AsyncSession,
    norad_id: int,
    line1: str,
    line2: str,
    epoch: datetime,
) -> None:
    """Insert a TLE row, ignoring duplicates on (norad_id, epoch).

    Historical rows are never deleted — this is an append-only table.
    """
    await session.execute(
        text(
            """
            INSERT INTO tles (norad_id, line1, line2, epoch)
            VALUES (:norad_id, :line1, :line2, :epoch)
            ON CONFLICT (norad_id, epoch) DO NOTHING
            """
        ),
        {"norad_id": norad_id, "line1": line1, "line2": line2, "epoch": epoch},
    )
