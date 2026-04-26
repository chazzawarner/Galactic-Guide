"""Export the FastAPI OpenAPI schema to a JSON file.

This script is called by the ``types-codegen`` Docker service so that
``packages/types`` can generate TypeScript types from the live schema.

Usage::

    uv run --project apps/api python -m galactic_api.scripts.export_openapi
"""

from __future__ import annotations

import json
import os
from pathlib import Path


def main() -> None:
    """Write the OpenAPI JSON to the path given by ``OPENAPI_OUTPUT`` (or a default)."""
    # Set required env vars to dummy values if not already present so that the
    # FastAPI app can be imported without a live database/Redis.
    os.environ.setdefault("DATABASE_URL", "postgresql+asyncpg://dummy:dummy@localhost/dummy")
    os.environ.setdefault("REDIS_URL", "redis://localhost:6379/0")

    # Import *after* env vars are set so the engine factory does not blow up.
    from galactic_api.main import create_app  # noqa: PLC0415

    app = create_app()
    schema = app.openapi()

    output_path = Path(os.environ.get("OPENAPI_OUTPUT", "openapi.json"))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(schema, indent=2))
    print(f"OpenAPI schema written to {output_path}")  # noqa: T201


if __name__ == "__main__":
    main()
