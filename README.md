# Galactic Guide

A 3D satellite dashboard. Pick one of five well-known Earth-orbiting satellites,
watch its orbit on a textured globe, and fast-forward through time.

> **Status: documentation phase.** The repo currently holds v1 design docs in
> `/docs/` and a dormant Bevy solar-system viewer in `src/` (kept buildable, not
> shipped). Scaffolding for the Next.js dashboard, FastAPI service, and
> Rust + Nyx worker lands once the PRD and roadmap are confirmed.

## What's in this repo today

| Path                  | What it is                                                                  |
|-----------------------|-----------------------------------------------------------------------------|
| `docs/spec.md`        | What we're building (v1 product spec, acceptance criteria, accessibility). |
| `docs/architecture.md`| How it's built (polyglot monorepo, Postgres, Redis Streams, R3F).           |
| `docs/api.md`         | The v1 HTTP API contract.                                                   |
| `docs/testing.md`     | Test types, oracles, and CI gate ordering.                                  |
| `assets/textures/`    | Planet textures. `earth.png` is the primary v1 asset.                       |
| `Cargo.toml`, `src/`  | Dormant Rust/Bevy viewer; will move under `crates/viewer/` when scaffolding lands. |

## Documentation index

Read in order for full context, or jump straight to the one you need:

1. **[Product spec](./docs/spec.md)** — features, target user, accessibility, acceptance criteria.
2. **[Architecture](./docs/architecture.md)** — services, data path, database schema, monorepo layout.
3. **[API contract](./docs/api.md)** — endpoints, request/response shapes, error model.
4. **[Testing strategy](./docs/testing.md)** — unit / integration / contract / visual / a11y layers, CI gates.

PRD and roadmap are next; they'll land in `docs/prd.md` and `docs/roadmap.md`.

## Planned local-dev workflow

These commands are the v1 target — they don't all work yet because the monorepo
isn't scaffolded. Captured here for reference and so the PRD/roadmap can plan
against it.

### One-time toolchain

The happy path needs **only Docker** (Engine ≥ 24, Compose v2). Every app —
web, api, worker — runs in a container; Postgres and Redis are official
images. No host-side Bun, uv, or Rust toolchain is required to bring up the
stack.

```bash
docker --version          # ≥ 24
docker compose version    # v2
```

### From a fresh clone

```bash
git clone https://github.com/chazzawarner/Galactic-Guide.git
cd Galactic-Guide
cp .env.example .env                 # local overrides; .env.example is committed

docker compose up                    # whole stack: web :3000, api :8000, worker, postgres, redis
```

The first run builds the per-app images and applies Alembic migrations via a
one-shot `migrate` service before `api` starts. Subsequent `docker compose up`
invocations reuse the cached images and the named `pgdata` volume.

The dashboard will be at <http://localhost:3000>. See
[`docs/architecture.md` § Verification](./docs/architecture.md#verification-fresh-clone)
for the full smoke-test sequence and the per-service container topology.

<details>
<summary>Running outside Docker (advanced)</summary>

For native-speed iteration on a single app, you can install the host
toolchains and run that app directly while the rest of the stack stays in
containers:

```bash
curl -fsSL https://bun.sh/install | bash         # JS package manager (Bun ≥ 1.1.30)
curl -LsSf https://astral.sh/uv/install.sh | sh  # Python tooling (uv ≥ 0.5)
rustup show                                      # picks up rust-toolchain.toml

docker compose up -d postgres redis              # only the dependencies
bun install && uv sync && cargo build --workspace
uv run alembic -c apps/api/alembic.ini upgrade head
bun run dev                                      # web, api, worker on the host
```

This is opt-in. CI and the canonical contributor flow stay container-based.

</details>

### Common commands (planned)

Every gate runs inside a container so a laptop and CI execute the same
environment:

```bash
docker compose run --rm web    bun run lint        # biome/eslint, tsc
docker compose run --rm web    bun run test        # vitest
docker compose run --rm web    bun run typecheck   # tsc --noEmit
docker compose run --rm api    uv run ruff check
docker compose run --rm api    uv run mypy --strict .
docker compose run --rm api    uv run pytest
docker compose run --rm worker cargo fmt --check
docker compose run --rm worker cargo clippy --workspace -- -D warnings
docker compose run --rm worker cargo test --workspace
```

CI gate ordering, oracles, and what each layer covers live in
[`docs/testing.md`](./docs/testing.md).

## Branch convention

Active development happens on feature branches with the prefix `claude/...`
(e.g. `claude/satellite-dashboard-planning-…`). `main` only receives reviewed
merges. Never push directly to `main`.

## Document status legend

When reading the docs, treat them as:

- **confirmed** — locked; implementation must conform.
- **in progress** — currently being iterated; expect changes.
- **planned** — to be written after the PRD + roadmap land.
- **deferred** — explicitly out of v1 (called out in each doc's "not in v1" section).

## Repository scope and history

Galactic Guide started as a Rust + Bevy solar-system viewer. The scope has
narrowed to a focused web-first satellite dashboard; the viewer crate is kept
in the workspace so its Cargo build keeps passing during the migration but is
not part of the v1 product surface.
