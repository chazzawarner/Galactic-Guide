# Galactic Guide — Architecture (v1)

## Purpose

This document describes the technical architecture for the v1 satellite dashboard described in [`spec.md`](./spec.md). It defines the polyglot monorepo (TypeScript + Python + Rust), the data path from CelesTrak to the browser, the database schema, the job queue, the rendering strategy, and the things to verify before saying "v1 is done." It is the contract that scaffolding work in subsequent PRs will follow.

## System overview

```
 ┌────────────────┐  HTTP/JSON   ┌─────────────────┐  XADD jobs   ┌────────────────┐
 │ apps/web       │◀────────────▶│ apps/api        │─────────────▶│ apps/worker    │
 │ Next.js 15+R3F │  TanStack    │ FastAPI/Pydantic│              │ Rust + Nyx     │
 │ Tailwind v4    │  Query       │ + SQLAlchemy 2  │◀── PUBLISH ──│ (SGP4 from TLE)│
 │ shadcn, drei   │              └────────┬────────┘   results    └────────┬───────┘
 └────────────────┘                       │                                │
        │                                 ▼                                ▼
        │                       ┌───────────────────────────────────────────┐
        │                       │ Postgres 16  ── satellites, tles,         │
        │                       │                 propagated_windows        │
        │                       │ Redis 7      ── job streams + pubsub +    │
        │                       │                 5-min hot result cache    │
        │                       └───────────────────────────────────────────┘
        ▼
  packages/types  ◀── openapi-typescript ── openapi.json (exported from Pydantic)
```

Data flow at a glance:

1. The browser (`apps/web`) asks for a **trajectory window** for the selected satellite — a list of sampled positions (and velocities) covering the next hour at 10 s steps.
2. `apps/api` (FastAPI) handles the request. It checks Redis (5-min hot cache) → Postgres (`propagated_windows`) → and only on a miss enqueues a job on a Redis Stream and awaits a pubsub result.
3. `apps/worker` (Rust + Nyx) consumes the job, propagates with SGP4, writes the result to Postgres, and publishes it on the result channel.
4. The browser caches the window with TanStack Query and **interpolates between samples each frame** (cubic Hermite using returned velocities), prefetching the next window when the current one is 75% consumed.

Persistent storage exists primarily because **CelesTrak rate-limits us**, and secondarily because re-running the propagator for repeat views is wasteful. Postgres is the durable source of truth for satellites, TLE history, and propagation results; Redis is the hot path for both job queueing and short-lived result caching.

## Monorepo layout

```
Galactic-Guide/
├── docs/                       # Source of truth for spec, architecture, prd, roadmap
├── Cargo.toml                  # converted to [workspace]
├── package.json                # Bun workspaces root (apps/*, packages/*)
├── turbo.json                  # JS + Python + Rust task graph
├── pyproject.toml              # uv workspace (members = ["apps/api"])
├── rust-toolchain.toml         # stable
├── .python-version             # 3.12
├── docker-compose.yml          # redis + postgres for local dev
│
├── apps/
│   ├── web/                    # Next.js 15 dashboard
│   │   ├── app/(dashboard)/page.tsx
│   │   ├── components/{Globe,SatellitePanel,TimeControls}.tsx
│   │   ├── lib/{api,query-client,interpolate,gmst}.ts
│   │   ├── public/textures/earth.png   # copied from /assets/textures/earth.png
│   │   ├── app/globals.css             # @import "tailwindcss"; @theme {…}
│   │   └── components.json             # shadcn → @galactic/ui registry
│   ├── api/                    # FastAPI + Pydantic v2 + SQLAlchemy 2 + Alembic
│   │   ├── src/galactic_api/{main,routers/*,services/*,models/*,db/*}.py
│   │   ├── src/galactic_api/scripts/export_openapi.py
│   │   ├── alembic/                   # migrations
│   │   ├── data/celestrak-fallback.json
│   │   └── pyproject.toml
│   ├── worker/                 # Rust + Nyx propagator (workspace member)
│   │   ├── Cargo.toml
│   │   └── src/main.rs                # Redis Streams consumer loop
│   └── docs/                   # Deferred: Next.js + MDX renders /docs/*.md
│
├── crates/
│   └── viewer/                 # Legacy Bevy code; kept buildable
│
└── packages/
    ├── ui/                     # shadcn canonical workspace package (@galactic/ui)
    ├── types/                  # openapi-typescript output (@galactic/types)
    ├── tsconfig/
    └── eslint-config/
```

`Cargo.toml` at the repo root becomes a workspace with `members = ["apps/worker", "crates/viewer"]`. The legacy `src/` moves under `crates/viewer/src/` so `cargo test --workspace` keeps passing without shipping it to users.

`apps/docs` is **deferred for v1**. Until we wire up MDX rendering, `/docs/*.md` is the source of truth and is reviewed as plain Markdown in PRs.

## Tech stack & versions

| Layer | Choice | Notes |
|-------|--------|-------|
| JS package manager | Bun ≥ 1.1.30 | Workspaces, `bun run --filter` |
| Monorepo orchestrator | Turborepo 2.x | Tasks: `dev`, `build`, `lint`, `test`, `types#generate` |
| Web framework | Next.js 15 (App Router, RSC where useful) | React 19, Turbopack dev |
| Styling | Tailwind v4.1 (CSS-first via `@theme`) | No `tailwind.config.js`. Use `@source "../../packages/ui/src/**/*.tsx"` to scan workspace UI |
| Components | shadcn/ui in monorepo mode | `packages/ui` is the registry source; `apps/web/components.json` aliases `@galactic/ui` |
| 3D | three.js + @react-three/fiber + @react-three/drei | Earth radius = 1.0 scene unit |
| Data fetching | TanStack Query v5 | Cache key: `["trajectory", noradId, windowStartFloorMin, durationS, stepS, includeVelocity]` |
| Tables | TanStack Table v8 | Scaffolded; not on v1 critical path |
| API | FastAPI ~0.115 + Pydantic v2 | OpenAPI exported via script |
| ORM | SQLAlchemy 2.0 (async) + asyncpg | Alembic owns migrations |
| Python pkg manager | uv ≥ 0.5 | Workspace at root |
| Queue client (Py) | redis-py 5.x async | Streams + pubsub |
| Worker | Rust stable + Nyx 1.1.2 + `redis` 0.27 (`tokio-comp`,`streams`) + `sqlx` 0.8 | SGP4 from TLE for v1 |
| Database | Postgres 16 (Docker for dev) | satellites, tles, propagated_windows |
| Broker | Redis 7 (Docker for dev) | streams, pubsub, hot result cache |

### Network topology

`apps/web/next.config.ts` rewrites `/api/*` → `http://localhost:8000/v1/*` so the
browser only ever talks to a single origin (`localhost:3000`). CORS is therefore
not configured on FastAPI; the API is intended to be reached via the rewrite
proxy in dev and an equivalent reverse proxy in any future deployment. This is
why [`api.md`](./api.md) explicitly excludes a CORS spec.

## Database (Postgres)

CelesTrak rate-limits us, so we cache TLEs durably; we also cache propagation results so repeat views don't re-run Nyx. Three tables, all owned by Alembic migrations in `apps/api/alembic/`:

```sql
-- Curated v1 list. Seeded with the five satellites from spec.md.
CREATE TABLE satellites (
  id          SERIAL PRIMARY KEY,
  norad_id    INTEGER UNIQUE NOT NULL,
  name        TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('station','telescope','comms','gps','weather','other')),
  description TEXT,
  added_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Append-only TLE history. Old rows are kept for replay/audit.
CREATE TABLE tles (
  id          BIGSERIAL PRIMARY KEY,
  norad_id    INTEGER NOT NULL REFERENCES satellites(norad_id),
  line1       TEXT NOT NULL,
  line2       TEXT NOT NULL,
  epoch       TIMESTAMPTZ NOT NULL,            -- parsed from TLE
  fetched_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  source      TEXT NOT NULL DEFAULT 'celestrak',
  UNIQUE (norad_id, epoch)
);
CREATE INDEX tles_latest_idx ON tles (norad_id, epoch DESC);

-- Worker result cache: sampled trajectory windows.
CREATE TABLE propagated_windows (
  id            BIGSERIAL PRIMARY KEY,
  hash          TEXT UNIQUE NOT NULL,          -- sha256(tle_id, start_at, duration_s, step_s, frame, include_velocity)
  tle_id        BIGINT NOT NULL REFERENCES tles(id) ON DELETE CASCADE,
  start_at      TIMESTAMPTZ NOT NULL,
  duration_s    INTEGER NOT NULL,
  step_s        INTEGER NOT NULL,
  frame         TEXT NOT NULL,                 -- 'eci_j2000'
  include_velocity BOOLEAN NOT NULL DEFAULT TRUE,
  samples       JSONB NOT NULL,                -- [{t, r_km:[x,y,z], v_km_s:[vx,vy,vz]}, …]
  computed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX propagated_windows_lookup_idx ON propagated_windows (tle_id, start_at);
```

### Refresh & retention

- **TLE fetch.** APScheduler job in FastAPI runs every 6 hours. It pulls CelesTrak group endpoints (`stations`, `active`) plus targeted NORAD lookups for the curated five. It upserts new `(norad_id, epoch)` rows; old rows are retained for history.
- **Offline fallback.** A static snapshot lives at `apps/api/data/celestrak-fallback.json` and is used when CelesTrak is unreachable, when `OFFLINE=1` is set, or on the very first dev run before the fetch job has had a chance to fire.
- **Propagation cache.** The worker writes a row after each successful job. A nightly cleanup job deletes rows older than 7 days. A 5-minute Redis hot cache sits in front to skip the Postgres roundtrip on repeat hits.

### Read path with cache layering

1. Web → `GET /v1/satellites/{norad_id}/trajectory?from=…&duration=3600&step=10`.
2. FastAPI computes the window hash → checks Redis `cache:result:{hash}`. **Hit** → return immediately.
3. **Miss** → checks Postgres `propagated_windows` by hash. **Hit** → warm Redis, return.
4. **Miss** → look up the latest TLE for the satellite from `tles`, build a job message, `XADD stream:propagate`, await pubsub `result:{job_id}` with a 5 s timeout.
5. Worker propagates with Nyx, `INSERT INTO propagated_windows`, `PUBLISH result:{job_id}`, `XACK`.
6. FastAPI caches the result in Redis and returns it to the browser.

The worker writes to Postgres directly (via `sqlx`) because the result is already in-hand and a roundtrip via FastAPI would be wasteful. **Both services share the schema, but only Alembic in FastAPI owns migrations.** The worker uses `sqlx` with explicit column lists, not schema introspection, so a forgotten migration shows up as a clear compile-time error rather than a runtime surprise.

## Job queue (Redis Streams)

Redis is chosen over NATS for v1: it's a single dependency we want anyway (hot cache + queue), Streams give at-least-once delivery via consumer groups, and the Rust `redis` crate has solid Streams support.

- **`stream:propagate`** — request stream, consumer group `workers`.
- **`result:{job_id}`** — pubsub channel for low-latency response fanout.
- **`cache:result:{hash}`** — `SETEX` 5-minute hot cache (mirror of recent Postgres rows).

Message payload (JSON in stream field `payload`):

```json
{
  "job_id": "uuidv7",
  "kind": "propagate_window",
  "tle_id": 1234,
  "tle": {"line1": "...", "line2": "...", "name": "ISS"},
  "epoch": "2026-04-25T12:00:00Z",
  "duration_s": 3600,
  "step_s": 10,
  "frame": "eci_j2000",
  "include_velocity": true,
  "hash": "sha256:..."
}
```

`tle_id` rides along so the worker can persist results without re-querying Postgres. `hash` is the cache key derived from `(tle_id, start_at, duration_s, step_s, frame, include_velocity)`.

## Time-controlled propagation

Don't propagate per frame. The worker returns sampled windows; the browser interpolates.

- **Window.** 1 hour, step 10 s → 361 samples (inclusive of both endpoints) × `{pos, vel}` ≈ 50 KB JSON per satellite.
- **Prefetch.** When `simTime > windowStart + 0.75 * duration`, call `queryClient.prefetchQuery` for the next window. At 1000x speed, one hour of simulation runs in 3.6 s of wall time, so prefetch is critical.
- **Interpolation** (`apps/web/lib/interpolate.ts`):
  - **Default**: cubic Hermite (C1) using returned velocities. Far better than position-only interpolation at LEO step sizes (10 s ≈ 75 km of motion).
  - **Fallback**: centripetal Catmull-Rom (positions only).
- **Speed multiplier** changes only how `simTime` advances. Sample density and fetch logic do not change.
- **Five-marker fan-out.** On first paint the dashboard issues 5 parallel trajectory requests (one per curated satellite) for marker positions; only the selected satellite's window is used to draw the orbit polyline. TanStack Query deduplicates if the same window is requested twice in a render. Per [`spec.md`](./spec.md), v1 deliberately avoids a batch positions endpoint — five cached requests are cheap.

## Type safety end-to-end

1. Pydantic v2 models live in `apps/api/src/galactic_api/models/`.
2. `apps/api/src/galactic_api/scripts/export_openapi.py` writes `apps/api/openapi.json`.
3. `packages/types` runs `openapi-typescript ../../apps/api/openapi.json -o src/generated.ts`.
4. Turbo task graph: `web#dev` ⟵ `types#generate` ⟵ `api#export-openapi`.
5. A chokidar watcher in `packages/types` regenerates on `openapi.json` change in dev. An initial `generated.ts` stub is committed to avoid cold-start typecheck failures.
6. The web app consumes types via `import type { components } from "@galactic/types"` and a thin typed fetch wrapper (likely `openapi-fetch`).

## Coordinate frames & rendering

**Render in ECI (J2000); rotate the Earth mesh by GMST.**

- Nyx returns ECI state vectors natively. Converting to ECEF on the server is unnecessary work and would make orbit polylines slither across the screen as Earth turns.
- `apps/web/lib/gmst.ts` computes GMST from simulation time using the IAU 1982 model (~20 lines of code; we don't need the rest of `satellite.js`). It applies as a Y-axis rotation on the Earth mesh.
- Earth texture: copy `/home/user/Galactic-Guide/assets/textures/earth.png` → `apps/web/public/textures/earth.png` (real file, not a symlink — Next's static pipeline expects real files in `public/`).
- Internal scene units: Earth radius = 1.0; convert Nyx km via `/ 6378.137`.
- Axis mapping: Nyx Z<sub>eci</sub> → three.js Y. Document this in `lib/interpolate.ts`. Pick once, do not relitigate.

## Reusable assets

Copy at scaffold time into `apps/web/public/textures/`:

- `earth.png` (required).
- `moon.png` (future Moon overlay).
- `sun.png` (future Sun-direction billboard).

Other planet PNGs stay in `/assets` for later docs use.

## Risks & sharp edges

- **Tailwind v4 + monorepo content scanning.** Classes used in `packages/ui` are tree-shaken away unless `@source` directives include the workspace UI. Verify in `next build`, not just `next dev`.
- **shadcn workspace mode.** Package name (`@galactic/ui`) and `components.json` aliases must match across `apps/web` (and later `apps/docs`) and `packages/ui`. Mismatches cause silent dual installs.
- **Nyx + ANISE kernels.** v1 uses **SGP4 from TLE only** — no kernel downloads needed. `apps/worker/README.md` should call this out clearly so future contributors don't accidentally reach for high-fidelity propagation.
- **CelesTrak rate limits.** Primary mitigation is Postgres + 6 h fetch cadence. Plus 403/429 backoff and the committed offline-fallback snapshot.
- **Postgres + worker write coupling.** Both `apps/api` and `apps/worker` write to `propagated_windows`. Only Alembic (in FastAPI) owns migrations. The worker uses `sqlx` with explicit column lists, not runtime introspection.
- **Turbopack + Tailwind v4.** Stable on Next 15.1+; keep a `dev:webpack` fallback script in case of regressions.
- **OpenAPI cold-start race.** Types must exist before `web#dev` typechecks. Commit an initial `generated.ts` stub; Turbo `dependsOn` handles steady state.
- **Coordinate axis convention.** Nyx Z<sub>eci</sub> → three.js Y. Pick once, document once.
- **Legacy Bevy code.** Keeping `crates/viewer` in the workspace means `cargo test --workspace` continues to compile it. If it ever becomes maintenance burden, drop it from default workspace members and require `--package viewer` to opt in.
- **sqlx compile-time queries.** The worker uses `sqlx::query!` macros, which require either a live `DATABASE_URL` or the offline `sqlx-data.json` file at compile time. Commit `sqlx-data.json` and document the `cargo sqlx prepare` workflow in `apps/worker/README.md` so CI and fresh clones build without needing a running Postgres.

## Verification (fresh clone)

```bash
git clone … && cd Galactic-Guide

# one-time toolchain
curl -fsSL https://bun.sh/install | bash
curl -LsSf https://astral.sh/uv/install.sh | sh
rustup show                                  # picks up rust-toolchain.toml
docker compose up -d redis postgres          # docker-compose.yml ships in the repo

# install
bun install                                  # JS workspaces
uv sync                                      # Python workspace (apps/api)
cargo build --workspace                      # Rust worker + viewer

# database
uv run alembic -c apps/api/alembic.ini upgrade head

# dev (single command starts everything via Turbo)
bun run dev                                  # web :3000, api :8000, worker

# smoke
curl http://localhost:8000/v1/healthz
curl http://localhost:8000/v1/satellites
curl 'http://localhost:8000/v1/satellites/25544/trajectory?duration=3600&step=10' | jq '.samples | length'   # → 361
open http://localhost:3000

# CI gates
bun run lint && bun run test
cargo test --workspace
uv run pytest apps/api
```

A successful v1 means a fresh clone reaches a working ISS visualization with time controls in under 10 minutes.

## Next documents

- `docs/prd.md` — product requirements with concrete tickets, milestones, and an explicit cut line for v1 vs. later.
- `docs/roadmap.md` — milestone breakdown (M0 scaffold, M1 single-satellite end-to-end, M2 all five, M3 polish/perf).

These will be drafted after `spec.md` and `architecture.md` are confirmed.
