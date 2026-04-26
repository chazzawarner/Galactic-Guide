"""Tests for GET /v1/healthz.

Covers:
- Happy path: both Postgres and Redis healthy → 200 {"status": "ok", ...}
- Degraded: Postgres unreachable → 503 with postgres="fail", redis="ok"
- Degraded: Redis unreachable → 503 with redis="fail", postgres="ok"
"""

from __future__ import annotations

import pytest
from fastapi.testclient import TestClient
from testcontainers.postgres import PostgresContainer
from testcontainers.redis import RedisContainer


def _redis_url(rc: RedisContainer) -> str:
    host = rc.get_container_host_ip()
    port = rc.get_exposed_port(6379)
    return f"redis://{host}:{port}/0"


def test_healthz_ok(
    pg_container: PostgresContainer,
    redis_container: RedisContainer,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Both dependencies healthy → 200 with status='ok'."""
    monkeypatch.setenv("DATABASE_URL", pg_container.get_connection_url())
    monkeypatch.setenv("REDIS_URL", _redis_url(redis_container))
    monkeypatch.setenv("OFFLINE", "1")

    from galactic_api.main import create_app  # noqa: PLC0415

    app = create_app()
    with TestClient(app, raise_server_exceptions=False) as client:
        response = client.get("/v1/healthz")

    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "ok"
    assert body["checks"]["postgres"] == "ok"
    assert body["checks"]["redis"] == "ok"
    assert body["version"] == "0.1.0"
    assert "now" in body


def test_healthz_degraded_postgres(
    redis_container: RedisContainer,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Postgres unreachable → 503 with postgres='fail', redis='ok'."""
    # Port 9 (discard) is effectively never open.
    monkeypatch.setenv("DATABASE_URL", "postgresql+asyncpg://bad:bad@127.0.0.1:9/bad")
    monkeypatch.setenv("REDIS_URL", _redis_url(redis_container))
    monkeypatch.setenv("OFFLINE", "1")

    from galactic_api.main import create_app  # noqa: PLC0415

    app = create_app()
    with TestClient(app, raise_server_exceptions=False) as client:
        response = client.get("/v1/healthz")

    assert response.status_code == 503
    body = response.json()
    assert body["status"] == "degraded"
    assert body["checks"]["postgres"] == "fail"
    assert body["checks"]["redis"] == "ok"


def test_healthz_degraded_redis(
    pg_container: PostgresContainer,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Redis unreachable → 503 with redis='fail', postgres='ok'."""
    monkeypatch.setenv("DATABASE_URL", pg_container.get_connection_url())
    # Port 6378 should not be bound to anything.
    monkeypatch.setenv("REDIS_URL", "redis://127.0.0.1:6378/0")
    monkeypatch.setenv("OFFLINE", "1")

    from galactic_api.main import create_app  # noqa: PLC0415

    app = create_app()
    with TestClient(app, raise_server_exceptions=False) as client:
        response = client.get("/v1/healthz")

    assert response.status_code == 503
    body = response.json()
    assert body["status"] == "degraded"
    assert body["checks"]["redis"] == "fail"
    assert body["checks"]["postgres"] == "ok"
