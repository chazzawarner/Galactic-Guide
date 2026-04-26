# ---------------------------------------------------------------------------
# apps/web/Dockerfile
# Multi-stage build for the Next.js dashboard.
# Targets: deps, production-builder, production, dev
# ---------------------------------------------------------------------------

# ── deps ─────────────────────────────────────────────────────────────────────
FROM oven/bun:1.1 AS deps

WORKDIR /app

# Copy workspace manifests for dependency caching
COPY package.json bun.lockb* ./
COPY apps/web/package.json apps/web/
COPY packages/tsconfig/package.json packages/tsconfig/
COPY packages/eslint-config/package.json packages/eslint-config/
COPY packages/ui/package.json packages/ui/
COPY packages/types/package.json packages/types/

RUN bun install --frozen-lockfile

# ── production builder ────────────────────────────────────────────────────────
FROM deps AS production-builder

# Copy all source
COPY apps/web apps/web
COPY packages packages

# Copy generated types stub (replaced at runtime via types-codegen service)
COPY packages/types/src/generated.ts packages/types/src/generated.ts

WORKDIR /app/apps/web
RUN bun run build

# ── production ───────────────────────────────────────────────────────────────
FROM node:22-alpine AS production

WORKDIR /app

ENV NODE_ENV=production

COPY --from=production-builder /app/apps/web/.next/standalone ./
COPY --from=production-builder /app/apps/web/.next/static ./.next/static
COPY --from=production-builder /app/apps/web/public ./public

EXPOSE 3000
CMD ["node", "server.js"]

# ── dev ──────────────────────────────────────────────────────────────────────
FROM deps AS dev

# Source is bind-mounted in docker-compose.override.yml
WORKDIR /app/apps/web

EXPOSE 3000
CMD ["bun", "run", "dev"]
