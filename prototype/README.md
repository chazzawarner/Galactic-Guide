# Galactic Guide — UI feel prototype

Throwaway sandbox for the open UX questions in [`docs/spec.md`](../docs/spec.md).
**Not part of the production monorepo.** See
[`docs/prototype-handoff.md`](../docs/prototype-handoff.md) for the brief.

## Stack

Vite + React 19 + TypeScript, Storybook (`@storybook/react-vite`), Tailwind v4,
`three` + `@react-three/fiber` + `@react-three/drei`. Mock ISS trajectory
generated once via Python `sgp4` and committed under `mocks/`.

## Run

```bash
cd prototype
npm install
npm run storybook        # :6006 — TimeControls, SatellitePanel, Globe, Dashboard
npm run dev              # :5173 — App.tsx renders the composed Dashboard
npm run build-storybook  # storybook-static/ for screenshots / async review
```

## Decisions

Append every UX call to [`../prototype-findings.md`](../prototype-findings.md).
That file is the only artifact the PRD will read.

## Regenerating mocks

```bash
pip install sgp4
python3 mocks/generate.py   # rewrites mocks/25544-trajectory.json
```

The TLE in `mocks/generate.py` is pinned so reruns are deterministic.

## Disposing

Once the PRD absorbs the findings: `git tag prototype-v0` then drop the
`prototype/ui-feel` branch. Do **not** merge to `main`.
