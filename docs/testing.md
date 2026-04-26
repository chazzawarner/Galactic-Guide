# Galactic Guide — Testing Strategy (v1)

## Purpose

This document defines the test types, tools, fixtures, and CI gates for v1. Its job
is to make the [`spec.md`](./spec.md) acceptance criteria *checkable* — every
criterion below maps to a specific test or class of tests. It is also the contract
that the PRD and roadmap will draw from when they cut v1 scope.

Companion documents: [`spec.md`](./spec.md) (what we're building),
[`architecture.md`](./architecture.md) (how it's built), and
[`api.md`](./api.md) (HTTP contract).

## Running tests

Every gate runs inside a container. The repo's `docker-compose.yml` defines
`web`, `api`, and `worker` services with all toolchains baked in (see
[`architecture.md` § Container topology](./architecture.md#container-topology)).
Contributors and CI invoke gates the same way:

```bash
docker compose run --rm web    bun run lint
docker compose run --rm web    bun run test
docker compose run --rm api    uv run pytest
docker compose run --rm worker cargo test --workspace
```

CI uses the production-shaped builds via `docker compose -f docker-compose.yml
-f docker-compose.ci.yml run --rm <svc> <cmd>` so image layers are
deterministic and there are no source bind-mounts. A laptop with only Docker
installed can reproduce any CI failure with the same command.

**Containers vs. testcontainers.** Integration suites still use
`testcontainers-python` and `testcontainers-rs` to spin up *throwaway*
Postgres + Redis pairs per test class — that gives hermetic state and
parallel safety, which the long-lived dev compose stack does not. Treat the
project compose file as the **runner** (where the test command executes) and
testcontainers as the **fixture** (where stateful dependencies live). The
one exception is the worker integration suite — see *Worker integration
fixture* under `apps/worker` below.

## Principles

- **Fast feedback over coverage theatre.** Unit tests run in seconds; integration
  in tens of seconds; visual smoke in under two minutes. We do not chase a coverage
  number, but each PR-touched file should have at least one test.
- **Deterministic by default.** No live network, no wall-clock dependencies, no
  uninitialized random seeds. Time is frozen, TLEs come from fixtures, GPU work
  in CI runs on a known Linux container.
- **One canonical oracle per claim.** SGP4 propagation is verified against the
  Python `sgp4` package, full stop. Visual checks against committed snapshots, not
  ad-hoc screenshots.
- **Prefer behavioural assertions over pixel diffs.** Canvas screenshots are
  flaky across drivers; assert on DOM, marker count, panel text first; pixel-diff
  is a stretch goal.
- **No flakes tolerated.** A test that fails intermittently is treated as a
  red test until quarantined or fixed. We do not retry-and-pretend.

## Test types at a glance

| Layer | Where | Tool | Speed | Runs in CI? |
|-------|-------|------|-------|-------------|
| Lint / format | all apps | ruff, biome (or eslint), clippy, `cargo fmt --check` | <5 s | Yes — gate 1 |
| Type-check | all apps | mypy (Python), tsc (web), `cargo check` | <30 s | Yes — gate 1 |
| Unit | per app | pytest, vitest, `cargo test` | <10 s/app | Yes — gate 2 |
| Integration | api, worker | pytest + testcontainers-python; Rust + testcontainers-rs | <60 s | Yes — gate 3 |
| Propagation accuracy | worker | pytest harness driving the Rust binary | <30 s | Yes — gate 3 |
| API drift check | api + types | `scripts/check-api-contract.py` | <5 s | Yes — gate 4 |
| Component / hook | web | vitest + @testing-library/react | <20 s | Yes — gate 2 |
| Visual smoke | web | Playwright (Chromium, Linux container) | <120 s | Yes — gate 5 |
| Accessibility audit | web | Playwright + `@axe-core/playwright`; `prefers-reduced-motion` vitest | <30 s | Yes — gate 5 |
| Contract response validation | api | pytest + `jsonschema` | <30 s | **M2, not v1** |
| Performance / load | api, worker | k6 / Criterion bench | minutes | **M2, not v1** |

## Per-app testing

### `apps/api` (FastAPI + Pydantic + SQLAlchemy)

**Unit (`apps/api/tests/unit/`)**

- TLE parser / classical-element derivation. Fixed TLE inputs → assert
  `(a, e, i, RAAN, ω, M, period)` to 4 sig figs against values produced by the
  `sgp4` reference (one-time computed and committed alongside the test).
- Hash function (`hash(tle_id, start_at, duration_s, step_s, frame, include_velocity)`)
  for cache keys — assert stability and determinism.
- Window validation logic (the rules from [`api.md`](./api.md): 60 ≤ duration ≤ 86400,
  1 ≤ step ≤ 600, etc.) — table-driven `pytest.mark.parametrize` test for every
  boundary.
- Time parsing — RFC 3339 with `Z` accepted, naive timestamps rejected.

Tools: `pytest`, `pytest-asyncio` (for async helpers), `freezegun` for time freezing,
`hypothesis` for property tests on the parser.

**Integration (`apps/api/tests/integration/`)**

- Full FastAPI app spun up in-process with `httpx.AsyncClient` against a
  Postgres + Redis pair from `testcontainers-python`. Alembic upgrades to head
  before each test class.
- Seed: the curated 5 satellites + a committed TLE snapshot for each.
- Cases:
  - `GET /v1/healthz` — 200 with both checks `ok`.
  - `GET /v1/healthz` — 503 when Redis container is paused.
  - `GET /v1/satellites` — exactly 5 entries, order stable.
  - `GET /v1/satellites/25544` — 200 with the seeded TLE; ETag set; second call
    with `If-None-Match` returns 304.
  - `GET /v1/satellites/99999` — 404 `satellite_not_found`.
  - `GET /v1/satellites/25544/trajectory` — 361 samples; `cached: false` first
    call, `cached: true` second call (Redis hit).
  - `GET /v1/satellites/25544/trajectory?duration=50` — 400 `invalid_window`
    with `details.duration` populated.
  - `GET /v1/satellites/25544/trajectory` with worker container stopped —
    504 `propagation_timeout` after 5 s.

The worker is run as a real container in this layer so we exercise the actual
Redis Streams round-trip, not a stub. Coordination uses a shared bridge network.

**Note on contract tests.** Validating *every* example response in
[`api.md`](./api.md) against the live API with `jsonschema` is deferred to **M2**
per [`api.md` § Drift detection](./api.md#drift-detection). v1 ships the
endpoint-table drift check (gate 4) only.

### `apps/worker` (Rust + Nyx + sqlx)

**Unit (`cargo test`)**

- Message-payload (de)serialization round-trip.
- Hash equality with the FastAPI implementation (golden vector committed
  alongside both implementations: identical inputs → identical hex digest).
- Sample-window builder: given a propagator output, assert sample count,
  monotonic `t`, and `t` always a multiple of `step_s`.

Tools: `cargo test` + `proptest` for property tests on the sample-window builder
(arbitrary `(duration, step)` in range → always `duration / step + 1` samples).

**Propagation accuracy (`apps/worker/tests/accuracy/`)**

- Golden vectors live at `apps/worker/tests/golden/sgp4/{norad_id}.json`. Each
  file is generated from the Python `sgp4` package using the same TLE the worker
  consumes, at fixed offsets `t ∈ {0, 60, 600, 3600}` seconds.
- The Rust test loads the TLE, runs Nyx's SGP4, and asserts position within
  **1 km** and velocity within **1 m/s** at each offset against the golden.
- Regeneration is gated: a `scripts/regen-goldens.py` script writes the JSON,
  but a reviewer must approve any diff. Goldens are not regenerated in CI.

**Integration (`apps/worker/tests/integration/`)**

- **Worker integration fixture (v1 baseline).** Use a shared Docker Compose
  fixture, not per-suite `testcontainers-rs`. `testcontainers-rs` against
  Postgres + Redis adds noticeable per-suite cold-start time on Linux CI;
  the gain in isolation isn't worth it for the worker, which doesn't write
  shared mutable state across tests beyond row-level idempotency it already
  asserts. The fixture is `docker compose -f docker-compose.yml -f
  docker-compose.ci.yml up -d postgres redis migrate` once per CI job.
  Tests run via `docker compose run --rm worker cargo test --workspace`
  against that running stack.
- Migrations are applied by the `migrate` one-shot service (Alembic is the
  single source of migrations per [`architecture.md`](./architecture.md);
  the worker never declares DDL).
- Each integration test publishes to `stream:propagate` and asserts that:
  - The worker `XREADGROUP`s and `XACK`s the message.
  - A row appears in `propagated_windows` with the expected `hash`.
  - A `PUBLISH` lands on `result:{job_id}` with the right payload.
- Idempotency: re-publishing the same job (same `hash`) must not duplicate
  the row. Tests reset `stream:propagate` and the relevant rows in
  `propagated_windows` between cases via SQL, not container restart.
- A test that genuinely needs an isolated DB (e.g. a destructive schema
  experiment) may opt into `testcontainers-rs` per case; mark such tests
  with `#[ignore = "isolated-db"]` and run them under a separate
  `cargo test --ignored` invocation in CI.

**sqlx offline mode.** `cargo sqlx prepare` runs in CI to refresh
`sqlx-data.json` against an ephemeral Postgres; the resulting file is committed
on PRs that change SQL. CI fails if the committed `sqlx-data.json` is stale.

### `apps/web` (Next.js + R3F)

**Unit (`apps/web/__tests__/unit/`)**

- `lib/interpolate.ts` — Hermite/Catmull-Rom against analytical answers on a
  circle: parameterize `(t)` over `[0, 1]`, assert reconstructed point lies on
  the unit circle within 1e-6.
- `lib/gmst.ts` — IAU 1982 GMST against published reference values from the
  Astronomical Almanac (one example per decade is enough).
- `lib/api.ts` typed-fetch wrapper — happy path + each error code from
  [`api.md`](./api.md) error table, with a mocked fetch.
- TanStack Query cache-key construction — assert the key includes
  `includeVelocity` and stable rounding of `windowStartFloorMin`.

Tools: `vitest`, `msw` for network mocking.

**Component / hook (`apps/web/__tests__/components/`)**

- `<TimeControls/>` — clicking pause sets the play state in one render;
  speed buttons update the multiplier; Now snaps `simTime` to a frozen `Date.now`.
- `<SatellitePanel/>` — given a fixture detail response, every field renders
  with the right unit suffix.
- `<Globe/>` — *behavioural* checks only: with a fixture trajectory store,
  the rendered DOM contains 5 `<canvas>`-marker accessibility hooks (one per
  satellite), the orbit polyline element exists for the selected satellite
  and not the others.

Tools: `@testing-library/react`, `vitest`. We do **not** snapshot the canvas
in unit tests — that's the visual smoke layer.

**Visual smoke (`apps/web/e2e/`)**

- Playwright on Chromium in a Linux container (`mcr.microsoft.com/playwright`)
  for determinism.
- Page object opens `/`, waits for the first frame to settle (signalled by a
  `data-ready="true"` attribute the dashboard sets when initial trajectories
  resolve), then asserts:
  - Globe canvas is non-blank (sample 8 corner pixels, expect ≥ 2 distinct
    colours).
  - Satellite panel contains "ISS" by default.
  - Speed buttons exist and respond to clicks (panel updates within one frame).
  - 1000x speed for 5 simulated seconds: the displayed UTC clock moves by ≥
    4500 simulated seconds and FPS counter (dev-only DOM element) reports
    ≥ 50.
- Pixel-diff snapshots are **not** v1. They land in M2 once the rendering is
  stable and we have a CI baseline.

### `packages/ui`

- shadcn components are upstream-tested. We add one vitest render assertion
  per component — "mounts without throwing, prop variants type-check." No
  Storybook in v1.

### `packages/types`

- Type-check only. No runtime tests. The `tsc --noEmit` step in gate 1 is the
  test.

## Cross-cutting tests

### API drift check (gate 4)

Per [`api.md` § Drift detection](./api.md#drift-detection), `scripts/check-api-contract.py`:

1. Generates `apps/api/openapi.json` from FastAPI's exporter.
2. Parses the endpoint table out of `docs/api.md`.
3. Asserts the set of `(method, path)` pairs matches exactly. Fails the
   build with a unified diff if not.

Lives at `scripts/check-api-contract.py`. Runs in CI gate 4.

### Field-level shape drift

Caught at compile time by `openapi-typescript`-generated types. A renamed Pydantic
field that the web app references becomes a `tsc` error in `apps/web`. No separate
test layer needed for this.

### Lint and type gates (gate 1)

- Python: `ruff check`, `ruff format --check`, `mypy --strict apps/api`.
- Rust: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`.
- JS/TS: `bun run lint` (biome or eslint), `tsc --noEmit` for each TS package.

These run in the first CI stage; the rest of the pipeline only proceeds on green.

## Reference oracles

The single source of truth for each numerical claim:

| Claim | Oracle |
|-------|--------|
| SGP4 position/velocity | `sgp4` Python package (Vallado-derived) |
| Classical orbital elements from TLE | `sgp4` Python package |
| GMST | IAU 1982 reference values (Astronomical Almanac) |
| Visual rendering (smoke only) | Playwright non-blank canvas check (≥ 2 distinct corner-pixel colours) on Chromium / Linux. **Not** pixel-diff snapshots — those are M2. |

`satellite.js` is **not** an oracle — it's a parity check (browser-side
implementation should agree with the API within wider tolerances). Diverging
values are investigated, not auto-trusted.

## Acceptance-criteria mapping

Every spec.md acceptance criterion has a home:

| AC | Test |
|----|------|
| 1 (ISS position within 0.1°) | `apps/worker/tests/accuracy/iss_test.rs` — assert against `sgp4` golden at `t=0`. Test bound (1 km / 1 m/s) is intentionally tighter than the 0.1° spec criterion (~12 km at LEO). |
| 2 (orbital elements to 4 sig figs) | `apps/api/tests/unit/test_elements.py` parametrised over all 5 satellites |
| 3 (≥50 fps at 1000x for 60 s) | Automated: Playwright dev-only FPS DOM check on a 5 s sample. Manual: full 60 s observation pre-release (same pattern as AC #6). Pixel-diff perf trace is M2. |
| 4 (geographic correctness) | `apps/web/__tests__/unit/gmst.test.ts` (vitest), plus a Playwright check that the ISS sub-satellite point lines up with a fixed-time fixture |
| 5 (controls within one frame) | `<TimeControls/>` vitest component test asserts state changes synchronously on click; React's synchronous re-render guarantee covers the "one frame" rendering claim, so no extra Playwright check is needed. |
| 6 (cold-start <10 min) | Documented in `docs/architecture.md` § Verification; not automated in v1 |
| 7 (offline development) | `apps/api/tests/integration/test_offline.py` runs the test suite with `OFFLINE=1` set |
| 8 (a11y & contrast) | Playwright + `@axe-core/playwright` audit on the loaded dashboard (no critical / serious violations); vitest test that `<TimeControls/>` honours `prefers-reduced-motion: reduce`; manual NVDA / VoiceOver smoke pre-release |

## CI gates

```
gate 1  lint + format + typecheck            (~30 s, parallel)
   ↓
gate 2  unit (apps/api, apps/web, apps/worker, packages/ui)   (parallel, ~60 s)
   ↓
gate 3  integration + propagation accuracy   (apps/api + apps/worker, ~90 s)
   ↓
gate 4  API drift check                      (<5 s)
   ↓
gate 5  Playwright visual smoke              (<120 s)
```

A red gate blocks merge. Gates within a layer run in parallel; layers run
serially so a fast lint failure aborts the rest.

Each gate is a `docker compose -f docker-compose.yml -f docker-compose.ci.yml
run --rm <svc> <cmd>` invocation, so the runner host only needs Docker and
nothing else. Concretely:

```
gate 1  docker compose run --rm web    bun run lint
        docker compose run --rm web    bun run typecheck
        docker compose run --rm api    uv run ruff check && uv run mypy --strict .
        docker compose run --rm worker cargo fmt --check && cargo clippy --workspace -- -D warnings
gate 2  docker compose run --rm web    bun run test
        docker compose run --rm api    uv run pytest tests/unit
        docker compose run --rm worker cargo test --workspace --lib
gate 3  docker compose run --rm api    uv run pytest tests/integration
        docker compose run --rm worker cargo test --workspace --test '*'
gate 4  docker compose run --rm api    python scripts/check-api-contract.py
gate 5  docker compose run --rm web    bun run test:e2e
```

CI runner: GitHub Actions on `ubuntu-24.04`. Gate 5 uses the prebuilt
Microsoft Playwright container *as the `web` test target*, so the browser
binary and OS deps are pinned and identical to the local invocation.

## Determinism playbook

- **Time.** `freezegun.freeze_time` in Python; `vi.useFakeTimers` in vitest;
  a `Clock` trait in Rust whose test impl returns a fixed `DateTime<Utc>`.
- **Network.** No live CelesTrak. The integration suite seeds Postgres from
  `apps/api/data/celestrak-fallback.json`. The web suite uses `msw` to mock
  the API.
- **Random seeds.** Rust `proptest` runs with `PROPTEST_CASES=64` and a
  pinned seed; failing seeds are recorded in `proptest-regressions/`.
- **GPU.** Visual smoke runs only on the official Playwright Linux image,
  invoked via `docker compose run --rm web bun run test:e2e`. The image
  pin lives in the `web` Dockerfile's e2e target, so a contributor's
  laptop and CI run the same browser and OS layer. Local runs against
  host browsers are not authoritative — only CI snapshots are.
- **DB ordering.** Every integration test that asserts list order also
  pins an `ORDER BY` in the query. No reliance on insertion order.

## What we deliberately don't test in v1

- **Load / soak.** No k6 or Locust runs. Five satellites and one user is the
  v1 target; perf headroom comes in M3.
- **Cross-browser.** Chromium only. Firefox / WebKit deferred to M2.
- **Mobile.** No mobile layouts in v1, so no mobile tests.
- **Auth flows.** No auth.
- **Pixel-diff visual snapshots.** Canvas snapshots come in M2 once the scene
  is stable.
- **Contract response validation against every api.md example.** Endpoint-table
  drift check is in v1; full schema validation is M2.

## Maintenance

- **Fixtures.** Shared test fixtures live at `apps/{api,web,worker}/tests/fixtures/`.
  The canonical TLE snapshot used by both the offline-fallback runtime path and
  the integration suite is `apps/api/data/celestrak-fallback.json` — single
  source of truth, not duplicated into per-test files.
- **Goldens.** `apps/worker/tests/golden/sgp4/*.json` are regenerated only via
  `scripts/regen-goldens.py`. Reviewers must approve any diff to a golden
  file in the same PR.
- **API examples.** Any change to a Pydantic response model must touch the
  matching example in [`api.md`](./api.md) and (in M2) the contract test fixture.
- **Flaky-test workflow.** A test that fails intermittently is quarantined
  within 24 hours via a skip marker (`pytest.mark.skip(reason=…)`,
  `it.skip(…)`, or `#[ignore]`) plus a tracking issue. The skip carries a
  7-day fix SLA; if unfixed by then, the test is reverted, not retried. There
  is no permanent "flaky" bucket and no auto-retry on CI.
- **Test ownership.** A failing test is owned by the author of the touched
  code, not the test author.

## Open questions

- **Visual baseline storage.** Commit Playwright screenshots to git, or store
  them in CI artefacts? Decision deferred — only matters in M2.
- **Property-testing budget.** `proptest` and `hypothesis` add CI time; cap at
  64 cases per property for v1, revisit if real bugs slip past.

(The previous open question on worker integration ergonomics is resolved
above: shared Docker Compose fixture is the v1 baseline, with opt-in
`testcontainers-rs` only for tests that genuinely need an isolated DB.)
