# Galactic Guide — Engineering Roadmap (v1)

**Status: confirmed**

Companion documents: [`prd.md`](./prd.md) (requirements & goals),
[`spec.md`](./spec.md) (feature spec & acceptance criteria),
[`architecture.md`](./architecture.md) (technical design),
[`api.md`](./api.md) (HTTP contract), [`testing.md`](./testing.md) (test strategy).

---

## Purpose

This document translates the PRD into concrete, sequenced implementation milestones. Each milestone has a clear scope, a list of deliverables, and a gate — a machine-checkable criterion that must be green before work on the next milestone begins. Milestones are ordered by dependency, not by calendar date. The roadmap covers v1 in full (Milestones 0–6) and lists the post-v1 backlog (M2+) for awareness.

> **Naming note.** Existing docs use "M2" to refer to features explicitly deferred past v1 (e.g. contract response validation, performance tests). This roadmap preserves that convention: everything within v1 is numbered M0–M6 as internal engineering phases, and "M2+" labels the post-v1 backlog. The two naming spaces do not overlap.

---

## Milestone overview

| # | Name | Gate summary |
|---|------|--------------|
| M0 | Docs & Design | PRD + roadmap confirmed; all companion docs confirmed |
| M1 | Monorepo Scaffolding | `cargo build --workspace`, `bun install`, `docker compose config` pass |
| M2 | Data Layer | `/v1/healthz` returns 200; five satellites seeded; fallback TLE loads |
| M3 | Worker (Rust + Nyx) | Propagation accuracy test passes (ISS within 0.1°) |
| M4 | API Endpoints | All five v1 endpoints green; contract drift check passes |
| M5 | Web Dashboard | Visual smoke + a11y audit pass; ≥ 50 fps at 1000× |
| M6 | v1 Verification & Hardening | All 8 ACs pass; all CI gates green on `main` |

---

## M0 — Docs & Design

**Goal:** Establish a confirmed design baseline so all subsequent implementation work has a locked specification to build against.

### Deliverables

- [ ] `docs/prd.md` confirmed (this document's companion).
- [ ] `docs/roadmap.md` confirmed (this document).
- [ ] `docs/spec.md` — already confirmed; no changes expected.
- [ ] `docs/architecture.md` — already confirmed; no changes expected.
- [ ] `docs/api.md` — already confirmed; no changes expected.
- [ ] `docs/testing.md` — already confirmed; no changes expected.

### Gate

Both `prd.md` and `roadmap.md` are merged to `main` and their status fields updated from **in progress** to **confirmed**. No open "confirmed" TODOs remain in any doc.

---

## M1 — Monorepo Scaffolding

**Goal:** Stand up the polyglot monorepo skeleton so every toolchain can build and the full Docker stack can start (even with empty application code).

### Deliverables

**Cargo workspace**
- Convert the root `Cargo.toml` from a single-package manifest to a `[workspace]` with `members = ["apps/worker", "crates/viewer"]`.
- Create `crates/viewer/` and move the existing `src/` → `crates/viewer/src/` and `src/bin/` → `crates/viewer/src/bin/`. Update `Cargo.toml` inside `crates/viewer/` accordingly.
- Create the `apps/worker/` skeleton with its own `Cargo.toml` (empty `main.rs` for now).
- Add `rust-toolchain.toml` (stable channel).
- Commit an initial `apps/worker/sqlx-data.json` stub so offline builds compile without a live Postgres.

**JS / Python workspace**
- Add root `package.json` with `"workspaces": ["apps/*", "packages/*"]` and the Turborepo dependency.
- Add `turbo.json` with the task graph: `dev`, `build`, `lint`, `test`, `typecheck`, `types#generate`.
- Add `pyproject.toml` (uv workspace root, `members = ["apps/api"]`).
- Add `.python-version` (`3.12`).
- Add `packages/tsconfig/` — shared base `tsconfig.json`.
- Add `packages/eslint-config/` — shared Biome / ESLint config.
- Add `packages/ui/` — empty shadcn monorepo registry package (`@galactic/ui`).
- Add `packages/types/` — `@galactic/types` with a stub `src/generated.ts` (empty export) so downstream type-checks don't fail cold.

**Docker stack**
- Add `docker-compose.yml` — base file, production Dockerfile targets, all seven services: `postgres`, `redis`, `migrate`, `api`, `types-codegen`, `worker`, `web`.
- Add `docker-compose.override.yml` — dev defaults: `dev` Dockerfile targets, bind-mounts, hot-reload, port exposure (`3000`, `8000`).
- Add `docker-compose.ci.yml` — CI overrides: production targets, no bind-mounts.
- Add `.env.example` — all required keys with safe placeholder values: `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `DATABASE_URL`, `REDIS_URL`, `OFFLINE`, `PROPTEST_CASES`.
- Add per-app `Dockerfile` stubs (multi-stage, `dev` + production targets) for `apps/api`, `apps/worker`, `apps/web`, and `packages/types`.

### Gate

All three of the following pass with no errors:

```bash
cargo build --workspace
bun install
docker compose config   # validates the compose YAML; no live containers needed
```

---

## M2 — Data Layer

**Goal:** Wire up the persistence layer (Postgres + Redis), the CelesTrak ingest script, the offline fallback, and a minimal FastAPI shell so the health endpoint returns 200 and all five satellites are seeded.

### Deliverables

**`apps/api` application skeleton**

Directory structure as specified in [`architecture.md` § Monorepo layout](./architecture.md#monorepo-layout):

```
apps/api/
├── src/galactic_api/
│   ├── main.py            # FastAPI app factory; mounts routers
│   ├── routers/
│   │   ├── healthz.py     # GET /v1/healthz
│   │   └── satellites.py  # stub — wired up fully in M4
│   ├── services/
│   │   ├── tle_ingest.py  # CelesTrak fetch + upsert
│   │   └── scheduler.py   # APScheduler job (6-hour TLE refresh)
│   ├── models/            # Pydantic v2 response models
│   ├── db/
│   │   ├── session.py     # async SQLAlchemy engine + session factory
│   │   └── queries.py     # typed query helpers
│   └── scripts/
│       └── export_openapi.py
├── alembic/               # migrations managed by Alembic
├── alembic.ini
├── data/
│   └── celestrak-fallback.json   # committed TLE snapshot
└── pyproject.toml
```

**Database migrations (Alembic)**
- Migration 001: create `satellites` table and seed the five curated rows (NORAD IDs, names, kinds per [`spec.md` § Curated satellite dropdown](./spec.md#curated-satellite-dropdown)).
- Migration 002: create `tles` table + `tles_latest_idx`.
- Migration 003: create `propagated_windows` table + `propagated_windows_lookup_idx`.

Schema matches exactly what is specified in [`architecture.md` § Database](./architecture.md#database-postgres).

**CelesTrak ingest**
- `tle_ingest.py` fetches TLEs for the curated five on startup and every 6 hours via APScheduler.
- Uses the offline fallback (`OFFLINE=1` or CelesTrak unreachable or no rows in `tles` yet).
- Upsert on `(norad_id, epoch)` — never deletes historical rows.
- `data/celestrak-fallback.json` committed with at least one recent TLE per satellite.

**Redis job queue wiring**
- Redis client initialised; connection health surfaced in `/v1/healthz`.
- `stream:propagate` consumer group `workers` created on startup (idempotent `XGROUP CREATE … MKSTREAM`).
- Worker job message schema (as in [`architecture.md` § Job queue](./architecture.md#job-queue-redis-streams)) serialised and sent by the API; deserialised by the worker in M3.

**`GET /v1/healthz`**
- Returns 200 `{"status":"ok","checks":{"postgres":"ok","redis":"ok"},"version":"…","now":"…"}` when both dependencies are healthy.
- Returns 503 `{"status":"degraded",…}` when either is unreachable.
- No caching headers (per [`api.md`](./api.md)).

### Gate

```bash
docker compose up postgres redis migrate
```

exits cleanly (all three services healthy or completed), and:

```bash
curl http://localhost:8000/v1/healthz
# → {"status":"ok","checks":{"postgres":"ok","redis":"ok"},…}
```

Plus: all five satellites are present in the `satellites` table, and at least one TLE row exists per satellite (loaded from the fallback).

---

## M3 — Worker (Rust + Nyx SGP4)

**Goal:** Implement the Rust propagation worker so it consumes jobs from `stream:propagate`, runs SGP4 via Nyx, writes the result to `propagated_windows`, and publishes back on the result channel.

### Deliverables

**`apps/worker/src/main.rs`**
- Redis Streams consumer loop using `XREADGROUP` against group `workers` on `stream:propagate`.
- Deserialise job payload (matching the schema in [`architecture.md` § Job queue](./architecture.md#job-queue-redis-streams)).
- Call Nyx's SGP4 propagator with the TLE and window parameters.
- Build the `samples` array: `duration_s / step_s + 1` samples, each `{t, r_km:[x,y,z], v_km_s:[vx,vy,vz]}` in ECI J2000.
- `INSERT INTO propagated_windows` via `sqlx` (explicit column list; no schema introspection). Uses `ON CONFLICT (hash) DO NOTHING` for idempotency.
- `PUBLISH result:{job_id}` with the full result payload.
- `XACK stream:propagate workers {message_id}`.
- `SQLX_OFFLINE=true` compile path supported via committed `sqlx-data.json`.

**Golden vectors & accuracy test**

- `scripts/regen-goldens.py` generates `apps/worker/tests/golden/sgp4/{norad_id}.json` for each of the five curated satellites from the Python `sgp4` package at fixed offsets `t ∈ {0, 60, 600, 3600}` seconds.
- `apps/worker/tests/accuracy/` Rust tests load each golden file, run Nyx SGP4 against the same TLE, and assert position within **1 km** and velocity within **1 m/s** at each offset. The 1 km / 1 m/s bound is intentionally tighter than AC #1's 0.1° angular-separation criterion (0.1° ≈ 12 km of arc at ISS altitude), so passing this test implies the AC is satisfied (per [`testing.md` § Acceptance-criteria mapping](./testing.md#acceptance-criteria-mapping)).
- Goldens are committed; regeneration requires a reviewer-approved PR diff. CI does not regenerate them.

**Unit tests**
- Message-payload (de)serialisation round-trip.
- Hash equality with the FastAPI implementation (golden vector: identical inputs → identical hex digest).
- Sample-window builder: assert sample count, monotonic `t`, `t` always a multiple of `step_s`.
- Property tests with `proptest`: arbitrary `(duration, step)` in valid range → always `duration / step + 1` samples.

**Integration tests**
- Fixture: `docker compose -f docker-compose.yml -f docker-compose.ci.yml up -d postgres redis migrate` (shared stack; not per-suite testcontainers-rs — see [`testing.md` § Worker integration fixture](./testing.md)).
- Assert: XREADGROUP → XACK; row appears in `propagated_windows` with correct `hash`; PUBLISH lands on `result:{job_id}`.
- Idempotency: re-publishing same job must not duplicate the row.

### Gate

```bash
docker compose run --rm worker cargo test --workspace
```

passes, including the propagation accuracy tests. The ISS position at `t = 0` (epoch) must be within **1 km** of the Python `sgp4` reference. A 1 km bound at epoch is sufficient coverage for AC #1 (0.1° ≈ 12 km at ISS altitude), as noted in [`testing.md` § Acceptance-criteria mapping](./testing.md#acceptance-criteria-mapping).

---

## M4 — API Endpoints

**Goal:** Implement the remaining four v1 API endpoints and generate the TypeScript types, so the web layer has a fully typed, tested API to call.

### Deliverables

**`apps/api` endpoint implementations**

Implement all five endpoints per the frozen contract in [`api.md`](./api.md):

| Endpoint | Notes |
|----------|-------|
| `GET /v1/healthz` | Already done in M2; verify it still passes. |
| `GET /v1/satellites` | Return all five rows; `Cache-Control: public, max-age=60`. |
| `GET /v1/satellites/{norad_id}` | Return TLE + derived classical elements; `ETag`; 304 on `If-None-Match` hit; `Cache-Control: public, max-age=300`. |
| `GET /v1/satellites/{norad_id}/tle` | Return latest TLE only; same caching as above. |
| `GET /v1/satellites/{norad_id}/trajectory` | Full cache layering (Redis → Postgres → worker job); all error codes from [`api.md`](./api.md#error-model). |

Error responses follow the `{detail, code, details?}` shape from [`api.md` § Error model](./api.md#error-model). Clients (and tests) must branch on `code`, not `detail`.

**OpenAPI export + type generation**
- `scripts/export_openapi.py` writes `apps/api/openapi.json` at API startup (also callable standalone).
- `packages/types/` runs `openapi-typescript ../../apps/api/openapi.json -o src/generated.ts`.
- Turbo task graph wires `web#dev` ← `types#generate` ← `api#export-openapi`.
- The `types-codegen` compose service watches the `openapi` shared volume in dev; runs once in CI.
- An initial `generated.ts` stub is committed to avoid cold-start typecheck failures.

**API tests (`apps/api/tests/`)**

Unit and integration cases as specified in [`testing.md` § apps/api](./testing.md):

- Unit: TLE parser, classical-element derivation, hash function, window validation (table-driven parametrize), time parsing.
- Integration (testcontainers-python): all cases from [`testing.md` § apps/api Integration`](./testing.md) — including 304, 404, 400 `invalid_window`, and 504 `propagation_timeout`.

**API contract drift check**
- `scripts/check-api-contract.py` asserts that the endpoint table in the live OpenAPI export matches the reference in `api.md`. Run as gate 4 in CI.

### Gate

```bash
docker compose run --rm api uv run mypy --strict .
docker compose run --rm api uv run pytest
```

both pass, and the contract drift check exits 0.

---

## M5 — Web Dashboard

**Goal:** Build the Next.js dashboard so all five satellites are visible on a 3D globe with working time controls, a detail panel, and a satellite dropdown — fully keyboard-accessible and meeting WCAG 2.1 AA.

### Deliverables

**`apps/web` application**

Directory structure per [`architecture.md` § Monorepo layout](./architecture.md#monorepo-layout):

```
apps/web/
├── app/(dashboard)/page.tsx
├── components/
│   ├── Globe.tsx          # R3F scene, Earth mesh, satellite markers, orbit polyline
│   ├── SatellitePanel.tsx # detail panel with orbital elements
│   └── TimeControls.tsx   # play/pause, speed, Now, UTC clock
├── lib/
│   ├── api.ts             # typed fetch wrapper (openapi-fetch + @galactic/types)
│   ├── query-client.ts    # TanStack Query client + prefetch logic
│   ├── interpolate.ts     # cubic Hermite (with velocity) + Catmull-Rom fallback
│   └── gmst.ts            # IAU 1982 GMST from simulation time
├── public/textures/
│   └── earth.png          # copied from assets/textures/earth.png (real file, not symlink)
└── app/globals.css        # @import "tailwindcss"; @theme { … }
```

**3D globe**
- React Three Fiber + drei; Earth radius = 1.0 scene unit.
- `earth.png` as texture (copy from `assets/textures/earth.png`; real file in `public/`).
- Earth mesh rotated by GMST (IAU 1982, computed from simulation time) around the Y-axis.
- Satellite markers at ECI J2000 positions (no ECEF conversion needed — ECI positions rendered directly; Earth mesh rotates under them).
- Orbit polyline for the selected satellite only (one full revolution).
- Mouse-orbit and zoom camera; up-axis fixed along the ecliptic normal; no free-tumble.

**Satellite dropdown**
- shadcn `<Select/>` component with ARIA semantics.
- Lists the five satellites by name; defaults to ISS on first visit.
- `localStorage` persistence: restores last-selected NORAD ID; falls back to ISS if the stored ID is not in the curated list.

**Detail panel**
- Displays all eight fields from [`spec.md` § Detail panel](./spec.md#detail-panel).
- Inline staleness notice when TLE epoch is older than 7 days.
- `<section aria-labelledby="…">` with satellite name as heading; announced by screen readers on selection change.

**Time controls**
- Play / pause toggle with `aria-pressed`.
- Speed buttons (1×, 10×, 100×, 1000×) with `aria-label`; `aria-pressed` on active speed.
- "Now" button; read-only UTC simulation-time display.
- `prefers-reduced-motion: reduce` → UI transitions disabled; simulation defaults to paused at 1×.

**Data fetching**
- TanStack Query v5; cache key: `["trajectory", noradId, windowStartFloorMin, durationS, stepS, includeVelocity]`.
- Five parallel trajectory requests on first paint (one per satellite) for marker positions.
- Prefetch next window when `simTime > windowStart + 0.75 * duration`.
- Cubic Hermite interpolation (`lib/interpolate.ts`) between samples using returned velocities. Centripetal Catmull-Rom fallback when velocities are absent.
- Typed fetch wrapper via `openapi-fetch` consuming `@galactic/types`.

**Accessibility (DOM controls)**
- Tab order: satellite dropdown → time controls → globe canvas (canvas skipped in v1 tab order).
- Space / Enter activates buttons; arrow keys cycle the dropdown; Esc closes the dropdown.
- No keyboard traps; every interactive control has a visible focus ring (≥ 2 px, 3:1 contrast).
- Textual fallback DOM elements mirror anything visible only on the globe.

**Web tests (`apps/web/__tests__/`)**

Per [`testing.md` § apps/web](./testing.md):

- Unit: `interpolate.ts` (circle round-trip), `gmst.ts` (Astronomical Almanac reference), `api.ts` (happy path + each error code), TanStack Query cache-key construction.
- Component: `<TimeControls/>`, `<SatellitePanel/>`, `<Globe/>` (behavioural checks; 5 marker hooks present, polyline for selected satellite only).
- Visual smoke: Playwright (Chromium, Linux container); committed snapshots.
- Accessibility: Playwright + `@axe-core/playwright`; `prefers-reduced-motion` vitest check.

### Gate

```bash
docker compose run --rm web bun run lint
docker compose run --rm web bun run typecheck
docker compose run --rm web bun run test
```

all pass, plus the Playwright visual smoke and accessibility audits. The dashboard renders at ≥ 50 fps at 1000× speed for 60 s (measured in the CI Chromium container).

---

## M6 — v1 Verification & Hardening

**Goal:** Verify that the integrated system meets all eight acceptance criteria, confirm CI is clean end-to-end, and close out Milestone 0 by marking all docs confirmed.

### Deliverables

**Full CI gate ordering**

Implement the five-gate CI pipeline from [`testing.md`](./testing.md) using `docker-compose.ci.yml`:

| Gate | What runs | Must be green before |
|------|-----------|---------------------|
| 1 | Lint + type-check (ruff, biome/eslint, clippy, `cargo fmt --check`, mypy, tsc) | Gate 2 |
| 2 | Unit tests (pytest, vitest, `cargo test`) | Gate 3 |
| 3 | Integration tests + propagation accuracy (testcontainers-python; Rust worker integration fixture) | Gate 4 |
| 4 | API contract drift check | Gate 5 |
| 5 | Visual smoke + accessibility audit (Playwright + axe-core) | — |

**Acceptance-criteria verification**

For each of the eight ACs in [`spec.md`](./spec.md) and [`prd.md`](./prd.md), confirm the covering test or check:

| AC | Covered by |
|----|-----------|
| 1 — ISS position accuracy < 0.1° | Worker propagation accuracy test: position within 1 km at `t=0` (epoch) vs. Python `sgp4` golden. The 1 km bound is intentionally tighter than 0.1° (≈ 12 km at ISS altitude). |
| 2 — Orbital elements to 4 sig figs | API unit test: TLE parser vs. `sgp4` reference |
| 3 — ≥ 50 fps at 1000× for 60 s | Playwright perf check in visual smoke suite |
| 4 — Geographic correctness ~1° | Visual smoke: known-time sub-satellite point check |
| 5 — Time control responsiveness within one frame | `<TimeControls/>` component tests |
| 6 — Cold-start < 10 min | Manual verification on a clean Docker environment; optionally timed in CI |
| 7 — Offline dev with fallback TLE | Integration test: `OFFLINE=1`, `/v1/satellites/25544` returns 200 with fallback data |
| 8 — Keyboard accessibility + contrast | `@axe-core/playwright` audit; NVDA / VoiceOver manual check |

**`sqlx-data.json` refresh**

Run `cargo sqlx prepare` against a live Postgres (the CI compose stack) and commit the result. CI fails if the file is stale after any SQL change.

**Doc confirmation**

Update status fields in `prd.md` and `roadmap.md` from **in progress** to **confirmed** in the PR that closes M6.

### Gate

All five CI gates pass on `main`. No open "confirmed" TODOs in any document. All eight acceptance criteria have a recorded passing test result or explicit sign-off.

---

## Post-v1 Backlog (M2+)

The following are explicitly deferred past v1. They are captured here for awareness and to prevent accidental over-engineering in v1 that blocks them later. A future PRD revision will define scope and sequencing.

| Item | Notes |
|------|-------|
| Contract response validation | `scripts/check-api-contract.py` extended to validate every example response in `api.md` against the live API using `jsonschema`. Currently gate 4 only checks the endpoint table. Deferred per [`api.md` § Drift detection](./api.md#drift-detection) and [`testing.md`](./testing.md). |
| Performance / load tests | k6 for the API; Criterion bench for the worker's propagation loop. Deferred per [`testing.md`](./testing.md). |
| Click-to-select on the globe | Globe marker is clickable; selection updates dropdown + panel. Explicitly deferred in [`spec.md` § What's not in v1](./spec.md#whats-not-in-v1). |
| Ground tracks | Sub-satellite point polyline on the Earth texture. Deferred. |
| Expanded satellite catalog & search | Browse / filter / search beyond the curated five. Deferred. |
| Mobile layout | Responsive design for phones and tablets. Deferred. |
| `apps/docs` MDX rendering | Next.js + MDX renders `docs/*.md` at `/docs/*`. Deferred per [`architecture.md`](./architecture.md). |
| Hosted / deployed version | Any cloud deployment. Requires auth review given the current no-auth assumption. |
