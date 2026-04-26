# Galactic Guide — Product Spec (v1)

Companion documents: [`architecture.md`](./architecture.md) (how it's built) and [`api.md`](./api.md) (HTTP contract).

## Overview

Galactic Guide is a 3D satellite dashboard that shows where a small set of well-known Earth-orbiting satellites are right now, with controls to fast-forward through time. The v1 experience is intentionally narrow: a textured Earth, a handful of carefully chosen satellites, an info panel, and a play/pause/speed widget. It is the smallest thing that demonstrates the full data path — TLE ingest, orbit propagation, type-safe API, and a 3D web client — and is the foundation we'll build a richer satellite catalog on later.

## Target user (v1)

- A developer or space-curious person running the project locally.
- Comfortable cloning a repo, running `bun run dev`, and opening `localhost:3000`.
- Not a satellite operator. Not on a phone. Not authenticated.

## What's in v1

### 3D globe

- A textured Earth rendered with React Three Fiber + drei, using the existing `assets/textures/earth.png`.
- Satellite markers drawn at their current positions.
- Orbit polylines drawn for the selected satellite (one full revolution).
- Smooth camera (mouse-orbit, zoom).

### Curated satellite dropdown

The v1 list is exactly five satellites, chosen to span common orbit families and be recognizable. NORAD IDs are pinned so behaviour is reproducible:

| Name | NORAD ID | Why it's in the list |
|------|----------|----------------------|
| ISS (ZARYA) | 25544 | The canonical "is the demo working" satellite — LEO, ~92 min period |
| Hubble Space Telescope | 20580 | LEO telescope; different inclination from ISS |
| Starlink-1007 | 44713 | Representative of the modern LEO mega-constellation |
| GPS BIIF-1 (NAVSTAR-65) | 36585 | MEO, semi-synchronous |
| NOAA-19 | 33591 | Sun-synchronous polar weather satellite |

(If a NORAD ID is decommissioned by the time we ship, it gets swapped for a live equivalent in the same category. The list is meant to stay at five.)

### Detail panel

When a satellite is selected, the side panel shows TLE-derived classical orbital elements, computed once per TLE refresh:

- Semi-major axis (a) in km
- Eccentricity (e)
- Inclination (i) in degrees
- Right ascension of ascending node (RAAN) in degrees
- Argument of perigee (ω) in degrees
- Mean anomaly (M) at epoch in degrees
- Orbital period in minutes
- Epoch (UTC, ISO 8601)

Plus the satellite's name and NORAD ID.

### Time controls

- Play / pause toggle.
- Speed multiplier with discrete steps: 1x, 10x, 100x, 1000x.
- "Now" button that snaps simulation time back to the wall clock.
- A read-only display of the current simulation time (UTC).

The simulation time drives both marker positions and the Earth's rotation (so the globe stays geographically correct as time advances).

## What's not in v1

- No catalog browse / search / filter — only the curated dropdown.
- No ground tracks (the line of sub-satellite points on Earth's surface).
- No pass predictions ("when does this fly over me?").
- No telemetry, status, or operator data.
- No authentication, user accounts, or saved state.
- No multi-user features.
- No mobile or tablet layout work — desktop only.
- No simultaneous orbit polylines — only the selected satellite gets a polyline; the other four are propagated for their marker positions only.

## Accessibility

Desktop-only in v1, so a11y commitments focus on keyboard operability, focus management, and contrast — not on full screen-reader narration of the 3D scene (canvas-based 3D is not natively accessible). The DOM controls around the canvas, however, must be fully usable without a pointer.

- **Keyboard navigation.**
  - Tab order: satellite dropdown → time controls (play/pause, 1x, 10x, 100x, 1000x, Now) → globe canvas (skipped from the tab order in v1; canvas keyboard interaction is M2).
  - Space / Enter activates buttons. Arrow keys cycle the dropdown. Esc closes the dropdown.
  - No keyboard traps.
- **Focus.** Every interactive control has a visible focus ring (≥ 2 px outline, 3:1 contrast against its background). No `:focus { outline: none }` without a replacement.
- **ARIA.** The dropdown uses the shadcn `<Select/>` (proxies native `<select>` ARIA semantics). Time-control buttons set `aria-pressed` on the active speed and carry `aria-label` for play/pause. The detail panel is a `<section aria-labelledby="…">` with the satellite name as its heading, so screen readers announce it on selection change.
- **Contrast.** Text and UI meet WCAG 2.1 AA: 4.5:1 for body text, 3:1 for large text and UI elements. The v1 palette is dark-mode only; any future light mode must meet the same bar.
- **Reduced motion.** When `prefers-reduced-motion: reduce` is set, the globe still rotates (it's the core feature), but UI transitions (dropdown open/close, panel slide-ins) are disabled and the simulation defaults to paused at 1x on first paint.
- **Textual fallback.** Anything visible only on the globe (selected satellite, current simulation time, marker count) is mirrored in DOM elements with semantic markup so a screen reader can read the dashboard's state even though it can't read the canvas.

Out of scope for v1: full screen-reader narration of orbital motion, voice control, customizable keybindings, canvas keyboard interaction.

## Acceptance criteria

A v1 build is considered shippable when:

1. **Position accuracy.** The ISS marker position at `t = now` is within 0.1° of angular separation from a reference SGP4 propagator (e.g. `sgp4` Python package or `satellite.js`) using the same TLE.
2. **Orbital elements correctness.** All six classical elements (a, e, i, RAAN, ω, M) plus the derived period match an independent TLE parser to 4 significant figures.
3. **Smoothness.** At 1000x speed, the globe maintains ≥ 50 fps on a mid-range laptop (M-series Mac or recent x86 with integrated graphics) for at least 60 seconds without dropped frames or visible marker stuttering.
4. **Geographic correctness.** With simulation time set to a known value, the sub-satellite point of the ISS lines up with that point on the Earth texture (within ~1° of arc, allowing for texture prime-meridian convention).
5. **Time control responsiveness.** Pressing pause stops marker motion within one frame. Changing speed takes effect within one frame. "Now" snaps within one frame.
6. **Cold-start time.** A fresh `git clone` to a working `localhost:3000` view (with all five satellites visible and selectable) completes in under 10 minutes on a developer machine with the prerequisites installed (Bun, uv, Rust, Docker).
7. **Offline development.** The dashboard works without internet access using a committed fallback TLE snapshot (no live CelesTrak fetch required for local dev).
8. **Keyboard accessibility & contrast.** All DOM controls are reachable in a logical tab order, operable via keyboard alone, and show a visible focus indicator. The detail panel is announced by NVDA / VoiceOver on selection change. Text and UI meet WCAG 2.1 AA. `prefers-reduced-motion: reduce` is honoured (no UI transitions; simulation defaults to paused at 1x).

## User flows (v1)

### First-time view
1. User opens `localhost:3000`.
2. Globe renders with all five satellites as markers; ISS is selected by default.
3. ISS orbit polyline is drawn.
4. Detail panel shows ISS orbital elements.
5. Time controls show "1x" and a live-updating UTC clock.

### Inspecting another satellite
1. User opens the satellite dropdown, picks Hubble.
2. Marker selection updates, orbit polyline redraws for Hubble.
3. Detail panel updates with Hubble's elements.

### Time travel
1. User clicks "1000x".
2. Markers begin rotating noticeably; Earth rotates underneath them.
3. User clicks pause; everything freezes mid-flight.
4. User clicks "Now"; simulation time snaps to wall clock; speed remains at 1000x but is paused.

## Open questions (resolve before PRD)

- **Selection method.** Should we **also** support click-to-select on the globe in v1, in addition to the dropdown?
- **Globe interaction.** Free-orbit camera, or constrained to "Earth always upright"?
- **Theme.** Dark-only for v1 (likely), or light/dark toggle?
- **Default satellite.** ISS (recognizable) or randomized from the five (avoids the appearance that ISS is special)?
- **What happens if a TLE is older than X days?** Show a staleness warning, or just propagate anyway?

## Glossary

- **TLE** — Two-Line Element set. The standard text format for distributing orbital data, primarily from CelesTrak / Space-Track.
- **SGP4** — The standard analytical propagator paired with TLEs. Good for ~1-week accuracy on LEO sats.
- **ECI (J2000)** — Earth-Centered Inertial frame at the J2000 epoch. The natural frame for orbits.
- **GMST** — Greenwich Mean Sidereal Time. Used to rotate the Earth model so geography lines up with ECI positions.
- **NORAD ID** — Five-digit catalog number that uniquely identifies a tracked object.
