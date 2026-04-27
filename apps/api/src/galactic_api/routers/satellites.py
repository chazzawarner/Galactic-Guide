"""Satellite routers — stub for M2; fully implemented in M4.

These placeholder routes exist so that the OpenAPI schema is populated from
the start and downstream consumers (types-codegen, web) can import it without
errors.
"""

from __future__ import annotations

from fastapi import APIRouter
from fastapi.responses import JSONResponse

router = APIRouter()


@router.get(
    "/satellites",
    summary="List curated satellites (M4)",
    tags=["satellites"],
    status_code=501,
)
async def list_satellites() -> JSONResponse:
    """Return the five curated satellites.  **Stub — implemented in M4.**"""
    return JSONResponse(
        status_code=501,
        content={"detail": "Not implemented yet — coming in M4", "code": "not_implemented"},
    )


@router.get(
    "/satellites/{norad_id}/trajectory",
    summary="Propagated trajectory window (M4)",
    tags=["satellites"],
    status_code=501,
)
async def get_trajectory(norad_id: int) -> JSONResponse:
    """Return a sampled trajectory window for one satellite.  **Stub — implemented in M4.**"""
    return JSONResponse(
        status_code=501,
        content={"detail": "Not implemented yet — coming in M4", "code": "not_implemented"},
    )
