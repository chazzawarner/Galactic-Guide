"""Pydantic v2 response model for the /v1/healthz endpoint."""

from __future__ import annotations

from pydantic import BaseModel


class Checks(BaseModel):
    """Per-dependency health check results."""

    postgres: str
    redis: str


class HealthResponse(BaseModel):
    """Response body for GET /v1/healthz."""

    status: str
    checks: Checks
    version: str
    now: str
