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

```bash
curl -fsSL https://bun.sh/install | bash         # JS package manager (Bun ≥ 1.1.30)
curl -LsSf https://astral.sh/uv/install.sh | sh  # Python tooling (uv ≥ 0.5)
rustup show                                      # picks up rust-toolchain.toml
# Docker required for Postgres + Redis containers
```

### From a fresh clone

```bash
git clone https://github.com/chazzawarner/Galactic-Guide.git
cd Galactic-Guide

docker compose up -d redis postgres        # infra
bun install                                # JS workspaces
uv sync                                    # Python workspace (apps/api)
cargo build --workspace                    # Rust workspace (apps/worker, crates/viewer)

uv run alembic -c apps/api/alembic.ini upgrade head    # database migrations

bun run dev                                # starts web :3000, api :8000, worker
```

The dashboard will be at <http://localhost:3000>. See
[`docs/architecture.md` § Verification](./docs/architecture.md#verification-fresh-clone)
for the full smoke-test sequence.

### Common commands (planned)

```bash
bun run lint           # biome/eslint, ruff, clippy, cargo fmt --check
bun run test           # vitest, pytest, cargo test
bun run typecheck      # tsc --noEmit, mypy --strict, cargo check
bun run build          # turbo build across all apps + packages
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
