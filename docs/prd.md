# Galactic Guide — Product Requirements Document (v1)

**Status: confirmed**

Companion documents: [`spec.md`](./spec.md) (detailed feature spec & acceptance criteria),
[`architecture.md`](./architecture.md) (how it's built), [`api.md`](./api.md) (HTTP contract),
[`testing.md`](./testing.md) (test strategy), and [`roadmap.md`](./roadmap.md) (implementation milestones).

---

## 1. Overview

Galactic Guide is a self-hostable, full-stack 3D satellite dashboard. It shows where a curated set of well-known Earth-orbiting satellites are right now and lets the viewer fast-forward through time on a textured globe. The v1 release is the smallest coherent product that exercises the complete data path — TLE ingest from CelesTrak, SGP4 orbit propagation in Rust, a type-safe HTTP API in Python, and a 3D React client — and forms the foundation for a richer satellite catalog in later versions. It doubles as a portfolio demonstration of polyglot full-stack engineering (TypeScript + Python + Rust) with a Docker-first contributor workflow.

---

## 2. Problem Statement

No simple, self-hostable project demonstrates the full satellite-tracking pipeline end-to-end. Existing tools are either hosted products (Heavens-Above, Celestrak web viewer) with no local-dev story, or individual scripts that skip the frontend, the persistence layer, or the type-safe API contract. Galactic Guide fills that gap: a single `docker compose up` brings up every layer, and contributors can see the whole chain working before they touch a line of code.

The secondary problem is learning friction. Working with TLEs, SGP4, ECI coordinates, and GMST conversion requires connecting concepts that are scattered across academic papers and dated tutorials. Galactic Guide provides a readable, documented reference implementation for each of those pieces inside a real product context.

---

## 3. Target Users (v1)

- **A developer or space-curious person** running the project locally.
- Comfortable cloning a repo and running `docker compose up` (or `bun run dev`) to open `localhost:3000`.
- Not a satellite operator. Not on a phone. Not authenticated.

There is exactly one v1 user persona. Multi-user features, authentication, and mobile layouts are out of scope until post-v1.

---

## 4. Goals & Non-Goals

### Goals

- Demonstrate the complete data path end-to-end: TLE ingest → SGP4 propagation → HTTP API → 3D web client.
- Ship five curated, recognizable satellites spanning common orbit families (LEO, MEO, sun-synchronous polar).
- Provide a Docker-first contributor experience: `docker compose up` is the single entry point; no host toolchain required.
- Commit a fallback TLE snapshot so the dashboard works offline and on a fresh clone without waiting for a live CelesTrak fetch.
- Meet all eight acceptance criteria listed in [`spec.md` § Acceptance criteria](./spec.md#acceptance-criteria) and reproduced in §6 below.
- Provide a foundation that post-v1 work (catalog browse, ground tracks, expanded satellites) can build on without re-architecting.

### Non-Goals (v1)

These are explicitly deferred and must not be designed against in v1 implementation work:

- **No catalog browse / search / filter** — only the curated dropdown of five.
- **No ground tracks** — the line of sub-satellite points on Earth's surface.
- **No pass predictions** — "when does this fly over me?"
- **No telemetry, status, or operator data.**
- **No authentication, user accounts, or saved state** (beyond `localStorage` for last-selected satellite).
- **No multi-user features.**
- **No mobile or tablet layout** — desktop only.
- **No simultaneous orbit polylines** — only the selected satellite gets a polyline.
- **No click-to-select on the globe** — selection is via the dropdown only.
- **No light theme or theme toggle** — dark-only.

---

## 5. Key Features (v1)

Full feature detail, acceptance criteria, and user flows live in [`spec.md`](./spec.md). This section is a concise summary for orientation.

### 3D Globe

A textured Earth rendered with React Three Fiber + drei, using `assets/textures/earth.png`. Satellite markers drawn at current positions. Orbit polyline drawn for the selected satellite (one full revolution). Smooth mouse-orbit and zoom camera; fixed up-axis along the ecliptic normal.

### Curated Satellite Dropdown

Exactly five satellites selected to span orbit families:

| Name | NORAD ID | Orbit family |
|------|----------|--------------|
| ISS (ZARYA) | 25544 | LEO station |
| Hubble Space Telescope | 20580 | LEO telescope |
| Starlink-1007 | 44713 | LEO mega-constellation |
| GPS BIIF-1 (NAVSTAR-65) | 36585 | MEO semi-synchronous |
| NOAA-19 | 33591 | Sun-synchronous polar |

### Detail Panel

TLE-derived classical orbital elements for the selected satellite: semi-major axis, eccentricity, inclination, RAAN, argument of perigee, mean anomaly, orbital period, and epoch. Inline staleness notice when the on-file TLE epoch is older than 7 days.

### Time Controls

Play / pause, discrete speed multiplier (1×, 10×, 100×, 1000×), "Now" button (snaps simulation time to wall clock), and a read-only UTC simulation-time display. The simulation time drives both marker positions and Earth rotation.

### Accessibility

All DOM controls around the canvas are keyboard-operable with logical tab order, visible focus rings (≥ 2 px, 3:1 contrast), and correct ARIA semantics. Text and UI meet WCAG 2.1 AA contrast. `prefers-reduced-motion: reduce` is honoured. See [`spec.md` § Accessibility](./spec.md#accessibility) for the full commitment.

---

## 6. Acceptance Criteria

A v1 build is considered shippable when all eight of the following criteria are met. These are reproduced verbatim from [`spec.md` § Acceptance criteria](./spec.md#acceptance-criteria); both documents own the list jointly. Any change to the list must be reflected in both files.

1. **Position accuracy.** The ISS marker position at `t = now` is within 0.1° of angular separation from a reference SGP4 propagator (e.g. `sgp4` Python package or `satellite.js`) using the same TLE.
2. **Orbital elements correctness.** All six classical elements (a, e, i, RAAN, ω, M) plus the derived period match an independent TLE parser to 4 significant figures.
3. **Smoothness.** At 1000× speed, the globe maintains ≥ 50 fps on a mid-range laptop (M-series Mac or recent x86 with integrated graphics) for at least 60 seconds without dropped frames or visible marker stuttering.
4. **Geographic correctness.** With simulation time set to a known value, the sub-satellite point of the ISS lines up with that point on the Earth texture (within ~1° of arc, allowing for texture prime-meridian convention).
5. **Time control responsiveness.** Pressing pause stops marker motion within one frame. Changing speed takes effect within one frame. "Now" snaps within one frame.
6. **Cold-start time.** A fresh `git clone` to a working `localhost:3000` view (with all five satellites visible and selectable) completes in under 10 minutes on a developer machine with the prerequisites installed (Docker Engine ≥ 24, Compose v2).
7. **Offline development.** The dashboard works without internet access using a committed fallback TLE snapshot (no live CelesTrak fetch required for local dev).
8. **Keyboard accessibility & contrast.** All DOM controls are reachable in a logical tab order, operable via keyboard alone, and show a visible focus indicator. The detail panel is announced by NVDA / VoiceOver on selection change. Text and UI meet WCAG 2.1 AA. `prefers-reduced-motion: reduce` is honoured (no UI transitions; simulation defaults to paused at 1× on first paint).

---

## 7. Technical Constraints

These constraints are architectural decisions already locked in [`architecture.md`](./architecture.md). They are captured here so the PRD is self-contained when read in isolation.

### Polyglot monorepo

| Layer | Technology | Notes |
|-------|-----------|-------|
| Web client | Next.js 15, React 19, Tailwind v4.1, R3F, shadcn/ui | App Router; Turbopack in dev |
| API | FastAPI ~0.115, Pydantic v2, SQLAlchemy 2 (async) | OpenAPI contract exported to `packages/types` |
| Worker | Rust stable + Nyx 1.1.2 | SGP4 propagation; Redis Streams consumer |
| Database | Postgres 16 | `satellites`, `tles`, `propagated_windows` tables |
| Broker / cache | Redis 7 | Job streams, pubsub result fanout, 5-min hot cache |
| JS tooling | Bun ≥ 1.1.30, Turborepo 2.x | Workspaces across `apps/*` and `packages/*` |
| Python tooling | uv ≥ 0.5, Python 3.12 | `uv` workspace at repo root |

### Docker-first local dev

The canonical contributor path requires only Docker Engine ≥ 24 and Compose v2. `docker compose up` starts the full stack. All CI gates run inside containers so a laptop and CI execute identical environments. See [`architecture.md` § Container topology](./architecture.md#container-topology).

### Offline fallback

`apps/api/data/celestrak-fallback.json` is committed and `COPY`-ed into the API image. The TLE schema it must satisfy is the TLE object shape defined in [`api.md` § `GET /v1/satellites/{norad_id}`](./api.md#get-v1satellitesnorad_id). The API uses the fallback when CelesTrak is unreachable, when `OFFLINE=1` is set, or on the first dev run before the background fetch job fires.

### Type safety end-to-end

Pydantic v2 models → `export_openapi.py` → `apps/api/openapi.json` → `openapi-typescript` → `packages/types/src/generated.ts`. The web client imports from `@galactic/types`; CI fails on drift between the live API and the committed schema.

### No CORS configuration

`apps/web/next.config.ts` rewrites `/api/*` → the internal FastAPI URL. The browser only ever talks to a single origin (`localhost:3000`), so FastAPI needs no CORS headers. Any future deployment must preserve an equivalent reverse-proxy setup.

---

## 8. Success Metrics

### Qualitative

- All eight v1 acceptance criteria (§6) pass with no exceptions or waivers.
- A contributor who has never seen the repo can run `docker compose up` and reach a working `localhost:3000` in under 10 minutes.
- The codebase passes all lint, type-check, and test gates in CI without manual intervention.

### Quantitative

| Metric | Target | Source |
|--------|--------|--------|
| ISS position accuracy | < 0.1° angular separation from `sgp4` reference | AC #1 |
| Orbital elements accuracy | 4 significant figures across all six elements | AC #2 |
| Rendering performance | ≥ 50 fps at 1000× for 60 s on a mid-range laptop | AC #3 |
| Cold-start time | < 10 minutes, fresh clone → working dashboard | AC #6 |
| Worker propagation latency | < 5 s end-to-end (enforced by the 5 s timeout in `api.md`) | API contract |

Post-v1 quantitative targets (performance/load tests, k6 benchmarks) are out of scope for v1 and explicitly deferred to M2+.

---

## 9. Open Questions & Assumptions

| # | Question | Assumption (until resolved) |
|---|----------|-----------------------------|
| 1 | What is the exact JSON schema for `celestrak-fallback.json`? | Deferred to the API contract. The fallback must produce TLE objects that conform to the shape in [`api.md` § `GET /v1/satellites/{norad_id}`](./api.md#get-v1satellitesnorad_id). The precise outer wrapper (array vs. keyed object) will be defined when the ingest script is written in Milestone 2. |
| 2 | What is the policy when a curated NORAD ID is decommissioned? | Swap the decommissioned satellite for a live equivalent in the same orbit-family category. The list stays at exactly five. The swap is a confirmed change requiring a PR that updates `spec.md`, `prd.md`, the seed data, and the fallback snapshot simultaneously. |
| 3 | Which features are in the post-v1 (M2+) round? | At minimum: full contract response validation (`jsonschema`), performance / load tests (k6, Criterion), click-to-select on the globe, ground tracks, and expanded satellite catalog. Mobile layout is explicitly deferred but not ruled out. The M2+ scope is not locked; it will be planned in a future PRD revision. |
| 4 | Will there ever be a hosted / deployed version? | Not in v1. The product is local-dev only. Any future deployment must respect the reverse-proxy constraint (§7) and will require a security review of the no-auth assumption. |

---

## 10. Document Status

| Document | Status |
|----------|--------|
| `docs/spec.md` | **confirmed** |
| `docs/architecture.md` | **confirmed** |
| `docs/api.md` | **confirmed** |
| `docs/testing.md` | **confirmed** |
| `docs/prd.md` (this file) | **confirmed** |
| `docs/roadmap.md` | **confirmed** |

This document moves to **confirmed** once the roadmap is confirmed and Milestone 0 is closed. Implementation must conform to any section marked **confirmed**; sections marked **in progress** may still change.

See the [README § Document status legend](../README.md#document-status-legend) for status definitions.
