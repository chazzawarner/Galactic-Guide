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

## Format for new entries

Append decisions in the form prescribed by the handoff:

> **Selection method.** Dropdown only.
>
> Reasoning: click-to-select on the globe required a custom raycasting layer
> that fought OrbitControls and felt fiddly; the dropdown was strictly better.
> M2 may revisit if a 50-sat catalog ever lands.
