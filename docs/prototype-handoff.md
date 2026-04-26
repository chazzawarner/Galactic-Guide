# Prototype Handoff — UI Feel & UX Open Questions

> **Process doc, not a v1 contract.** This briefs whoever picks up the
> throwaway prototype branch. It is *not* part of the production monorepo and
> should be archived (or deleted) once the PRD has absorbed its findings.

## Why we're prototyping

Several of [`spec.md`](./spec.md)'s "Open questions" are *feel* questions that
are faster to answer with a working sandbox than with more written debate:

- **Selection method** — dropdown only, or also click-to-select on the globe?
- **Globe interaction** — free-orbit camera, or constrained to "Earth always upright"?
- **Theme** — dark-only for v1, or light/dark toggle?
- **Default satellite** — ISS (recognizable) or randomized from the five?
- **TLE staleness** — show a banner when a TLE is older than N days, or propagate quietly?

Plus a few that haven't been written down yet but the prototype will surface:
panel placement (sidebar / drawer / overlay), marker style (sphere / billboard /
icon), orbit polyline thickness and colour, what 1000x speed actually *feels*
like.

The prototype is **timeboxed at two working sessions**. Its output is a
`prototype-findings.md` that becomes input to the PRD. The code does **not**
merge back to `main`.

## What's locked (don't relitigate)

Inputs from the v1 design docs. Treat as fixed:

- The five curated satellites and NORAD IDs — see [`spec.md`](./spec.md).
- Earth rendered in **ECI (J2000)** with the mesh rotated by GMST — see
  [`architecture.md` § Coordinate frames & rendering](./architecture.md#coordinate-frames--rendering).
- Detail panel fields: `a, e, i, RAAN, ω, M, period, epoch`.
- Time-control surface: play/pause, 1x / 10x / 100x / 1000x, Now.
- Trajectory data shape: 361 samples × `{t, r_km, v_km_s}` per window — see
  [`api.md`](./api.md).
- Earth texture is `assets/textures/earth.png`.

## What's unlocked (the point of the prototype)

Everything *visual*, *kinetic*, and *spatial* about the dashboard. Including:

- Camera control style and orbit polyline rendering.
- How the time controls feel at different speeds (do we actually want 1000x?).
- Panel layout: sidebar, drawer, overlay, bottom sheet.
- Marker style and orbit polyline aesthetics.
- Theme palette and typography.
- Anything else that "I'll know it when I see it."

## Stack — minimal and disposable

- **Vite + React 19 + TypeScript** for the dev server.
- **Storybook 8** for component-level prototyping.
- **@react-three/fiber + @react-three/drei + three.js** for the globe.
- **Tailwind v4** so the look-and-feel translates to v1.
- **shadcn/ui** installed locally so the primitives are the same as v1.

Explicitly **not** in this prototype: Next.js, FastAPI, Postgres, Redis,
testcontainers, the worker, any real propagation. Mock everything.

## Mock data

Two JSON fixtures committed alongside the prototype code (NOT in `/docs`):

- `prototype/mocks/satellites.json` — the five NORAD IDs and names from
  [`spec.md`](./spec.md).
- `prototype/mocks/{norad_id}-trajectory.json` — one real-shape trajectory
  window per satellite, generated once with the Python `sgp4` package and
  committed. 361 samples, step 10 s, includes velocity.

One trajectory file per satellite is enough — the speed multiplier just
advances `t` faster against the same window. The Hermite interpolator from the
v1 plan is small enough to bring along (~30 lines).

## What to build (in this order)

1. **`<TimeControls/>`** — pure DOM, no canvas. Fastest feedback loop. Test
   the 1000x feel against a fake `simTime` counter that advances at
   `speed * 16ms` per frame.
2. **`<SatellitePanel/>`** — pure DOM. Renders fixture data. Use it to nail
   theme, typography, and unit formatting.
3. **`<Globe/>`** — R3F scene with **one** ISS marker driven by the mock
   trajectory + a Hermite interpolator. Goal: prove the ECI/GMST setup feels
   right and the orbit polyline looks good. Earth texture comes from
   `/assets/textures/earth.png`.
4. **A composed dashboard story** — all three together. Try sidebar vs
   drawer vs overlay variants and screenshot each.

**Stop after step 4.** Don't add the other four satellites, don't wire up the
dropdown, don't build the prefetch logic. Those are v1's job.

## Decisions to capture

Append each decision (with a one-paragraph rationale) to
`prototype-findings.md` as you make it. Suggested format:

> **Selection method.** Dropdown only.
>
> Reasoning: click-to-select on the globe required a custom raycasting layer
> that fought OrbitControls and felt fiddly; the dropdown was strictly better.
> M2 may revisit if a 50-sat catalog ever lands.

By the end of the timebox, the file should answer every open question from
[`spec.md`](./spec.md) plus any new ones the prototype surfaced.

## Output

A pushed branch (or PR for review) containing:

1. The Storybook code — read-only reference for v1 implementers.
2. `prototype-findings.md` — the only artifact the PRD will actually consume.
3. Optional: short screen recordings of each story for asynchronous review.

Once the PRD absorbs the findings, archive the prototype:
`git tag prototype-v0` then drop the branch. Do **not** merge to `main`.

## Anti-goals

Do not, on this prototype:

- Set up the real monorepo layout, FastAPI, Rust worker, or any persistence.
- Build all five satellites simultaneously.
- Add real tests beyond what Storybook renders for free.
- Try to make it deployable or public.
- Run an a11y audit (a11y is locked in v1; auditing pre-PRD adds nothing).
- Optimise performance.
- Fix bugs in mocks or fixtures — regenerate them rather than patch.

## Suggested kickoff prompt

Paste this into a fresh Claude Code session on a new branch:

```
You are picking up the Galactic Guide UI prototype. Read these docs first
(they are inputs, not options): docs/spec.md, docs/architecture.md,
docs/api.md, docs/testing.md, docs/prototype-handoff.md.

Goal: a Storybook + Vite + React + R3F sandbox that answers the open UX
questions in spec.md. Throwaway code; do not touch the production monorepo
shape.

Stack: Vite, React 19, TypeScript, Storybook 8, @react-three/fiber, drei,
Tailwind v4, shadcn/ui. No Next.js, no backend.

Build in order: TimeControls (pure DOM), SatellitePanel (pure DOM), Globe
(R3F + Hermite interpolation against a mock trajectory window for ISS only),
then a composed dashboard story. Stop there.

Capture every UX decision in prototype-findings.md with a paragraph of
rationale. That file is the only artifact the PRD will read.

Two-session timebox. Do not merge to main.
```

## What happens after

You finish the prototype → write `prototype-findings.md` → we draft the PRD.
The PRD reads the findings, resolves the open questions in
[`spec.md`](./spec.md) (or pushes any back to v1.x), and commits scope.
After PRD: roadmap. After roadmap: code.
