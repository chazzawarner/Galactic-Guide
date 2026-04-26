# ---------------------------------------------------------------------------
# packages/types/Dockerfile
# Thin Bun image that generates TypeScript types from the OpenAPI spec.
# In dev: watches the shared openapi volume and regenerates on change.
# In CI:  runs once and exits.
# ---------------------------------------------------------------------------

FROM oven/bun:1.1 AS base

WORKDIR /app

COPY package.json bun.lockb* ./
COPY packages/types/package.json packages/types/
COPY packages/tsconfig/package.json packages/tsconfig/

RUN bun install --frozen-lockfile

COPY packages/types packages/types
COPY packages/tsconfig packages/tsconfig

WORKDIR /app/packages/types

# ── production (one-shot, used in CI) ────────────────────────────────────────
FROM base AS production

CMD ["bun", "run", "generate"]

# ── dev (watch mode) ─────────────────────────────────────────────────────────
FROM base AS dev

RUN bun add --dev chokidar-cli

# Watch the openapi volume mount for changes and regenerate
CMD ["bunx", "chokidar", "/openapi/openapi.json", "--initial", \
     "-c", "bun run generate"]
