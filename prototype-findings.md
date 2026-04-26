# Prototype Findings — UI Feel & UX Open Questions

> Draft. Updated as decisions are made during the two-session timebox per
> [`docs/prototype-handoff.md`](./docs/prototype-handoff.md). This file is the
> only artifact the PRD will read.

## Resolved (from `spec.md` "Open questions")

- **Selection method.** _Pending — evaluate dropdown vs click-to-select on the globe._
- **Globe interaction.** _Pending — free-orbit OrbitControls vs constrained "Earth always upright"._
- **Theme.** _Pending — dark-only for v1 or light/dark toggle._
- **Default satellite.** _Pending — ISS vs randomized from the five._
- **TLE staleness banner.** _Pending — banner threshold or propagate quietly._

## Surfaced by the prototype

- **Panel placement.** _Pending — Sidebar / Drawer / Overlay / BottomSheet variants live in `Dashboard.stories.tsx`. Pick one._
- **Marker style.** _Pending — sphere (current), billboard, or icon._
- **Orbit polyline aesthetic.** _Pending — colour, thickness, opacity, glow._
- **1000× feel.** _Pending — does it look smooth, dizzy, or broken?_

## Resolved (surfaced during scaffolding)

> **shadcn/ui.** Deferred until v1.
>
> Reasoning: the four prototype components (`TimeControls`, `SatellitePanel`,
> `Globe`, `Dashboard`) are small enough that a shadcn registry would be
> ceremonial — the only primitives that would have benefited are `Button` and
> `Card`, each used in one place. The handoff stack lists shadcn so v1
> primitives match, but the prototype's job is to surface UX questions and
> none of those questions require the canonical shadcn API. v1 should run
> `npx shadcn@latest init` against `packages/ui` as part of the real monorepo
> conversion. Custom Tailwind classes here use the `@theme` tokens declared in
> `prototype/src/index.css` so the palette transfers.

> **Earth texture orientation.** Verified analytically; visual check available.
>
> Reasoning: derivation chain (`SphereGeometry` UV layout + ECI/scene mapping
> + GMST sign) lands Greenwich at scene +X at GMST=0 — derivation comment in
> `Globe.tsx`. The `Globe → ReferenceMarkers` story renders red/green/blue
> markers parented to the Earth mesh at Greenwich, north pole, and the ±180°
> meridian; if the texture seam ever moves (or the source Earth PNG is
> swapped), open that story and confirm the markers land on the right
> coastlines before the orbit polyline can be trusted.

## Format for new entries

Append decisions in the form prescribed by the handoff:

> **Selection method.** Dropdown only.
>
> Reasoning: click-to-select on the globe required a custom raycasting layer
> that fought OrbitControls and felt fiddly; the dropdown was strictly better.
> M2 may revisit if a 50-sat catalog ever lands.
