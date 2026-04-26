# ---------------------------------------------------------------------------
# apps/api/Dockerfile
# Multi-stage build for the FastAPI service.
# Targets: base, dev, production, migrate
# ---------------------------------------------------------------------------

# ── base ─────────────────────────────────────────────────────────────────────
FROM python:3.12-slim AS base

WORKDIR /app

# Install uv
RUN pip install --no-cache-dir uv>=0.5

# Copy workspace & API package manifests first (better layer caching)
COPY pyproject.toml .python-version ./
COPY apps/api/pyproject.toml apps/api/

# ── production ───────────────────────────────────────────────────────────────
FROM base AS production

RUN uv sync --project apps/api --no-dev

COPY apps/api ./apps/api

CMD ["uv", "run", "--project", "apps/api", "uvicorn", "galactic_api.main:app", \
     "--host", "0.0.0.0", "--port", "8000"]

# ── dev ──────────────────────────────────────────────────────────────────────
FROM base AS dev

RUN uv sync --project apps/api

COPY apps/api ./apps/api

CMD ["uv", "run", "--project", "apps/api", "uvicorn", "galactic_api.main:app", \
     "--reload", "--host", "0.0.0.0", "--port", "8000"]

# ── migrate ──────────────────────────────────────────────────────────────────
FROM base AS migrate

RUN uv sync --project apps/api --no-dev

COPY apps/api ./apps/api

CMD ["uv", "run", "--project", "apps/api", \
     "alembic", "-c", "apps/api/alembic.ini", "upgrade", "head"]
