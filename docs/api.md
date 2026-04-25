# Galactic Guide — API Contract (v1)

## Status

This contract is **frozen for v1** before code is written. The FastAPI implementation
in `apps/api` and the TypeScript types generated into `packages/types` must conform
to it. Drift is caught by the CI check described in [Drift detection](#drift-detection).

Companion documents: [`spec.md`](./spec.md) (what we're building) and
[`architecture.md`](./architecture.md) (how it's built).

## Conventions

- **Base URL.** `http://localhost:8000` in dev. All endpoints are prefixed with `/v1`.
- **Versioning.** Path-based (`/v1/...`). Breaking changes bump to `/v2`. v1 may add
  *new* fields and endpoints without bumping; clients must ignore unknown fields.
- **Content type.** `application/json; charset=utf-8` for both requests and responses.
- **Time format.** RFC 3339 / ISO 8601 in UTC with a trailing `Z` (e.g.
  `2026-04-25T12:00:00Z`). The server rejects timestamps without a timezone.
- **Units.**
  - Distances: kilometers (km).
  - Velocities: kilometers per second (km/s).
  - Angles: degrees.
  - Durations / steps: seconds.
  - Periods: minutes (matches conventional satellite reporting).
- **Frame.** All position/velocity vectors are in **ECI J2000** unless an endpoint
  explicitly states otherwise. Vectors are 3-element arrays `[x, y, z]`.
- **IDs.** Satellites are addressed by their **NORAD catalog number** (integer) in
  URLs, not by the internal `id` primary key.

## Endpoints

| Method | Path                                      | Purpose                                  |
|--------|-------------------------------------------|------------------------------------------|
| GET    | `/v1/healthz`                             | Liveness + readiness                     |
| GET    | `/v1/satellites`                          | List the curated satellites              |
| GET    | `/v1/satellites/{norad_id}`               | Detail for one satellite (TLE + elements)|
| GET    | `/v1/satellites/{norad_id}/tle`           | Latest TLE only (debugging convenience)  |
| GET    | `/v1/satellites/{norad_id}/trajectory`    | Sampled propagation window               |

### `GET /v1/healthz`

Liveness + readiness probe. Returns 200 when the API process is up *and* it can reach
both Postgres and Redis. Used by Docker health checks and the dashboard's startup
guard.

**Response 200**
```json
{
  "status": "ok",
  "checks": {
    "postgres": "ok",
    "redis": "ok"
  },
  "version": "0.1.0",
  "now": "2026-04-25T12:00:00Z"
}
```

**Response 503** — same shape with `"status": "degraded"` and at least one check set
to `"fail"`. Always 503, never 500, so monitoring can distinguish "process died" from
"dependency down".

No caching headers.

### `GET /v1/satellites`

Returns the curated v1 list. There are exactly five satellites; pagination is
not implemented and clients must not assume it exists.

**Response 200**
```json
{
  "satellites": [
    {
      "norad_id": 25544,
      "name": "ISS (ZARYA)",
      "kind": "station",
      "description": "International Space Station"
    },
    {
      "norad_id": 20580,
      "name": "Hubble Space Telescope",
      "kind": "telescope",
      "description": null
    }
  ]
}
```

**Field constraints**

| Field         | Type    | Notes                                                        |
|---------------|---------|--------------------------------------------------------------|
| `norad_id`    | integer | Always > 0. Stable across versions.                          |
| `name`        | string  | Display name; 1–80 chars.                                    |
| `kind`        | enum    | One of `station`, `telescope`, `comms`, `gps`, `weather`, `other`. |
| `description` | string \| null | May be null for the leanest entries.                  |

**Caching.** `Cache-Control: public, max-age=60`. The list is curated and rarely
changes; 60 s lets the dashboard get fresh data quickly without hammering the API.

### `GET /v1/satellites/{norad_id}`

Detail for one satellite, including the latest TLE on file and the derived classical
elements that populate the dashboard's detail panel.

**Path parameter**

| Name       | Type    | Constraint        |
|------------|---------|-------------------|
| `norad_id` | integer | Must be > 0; must match a row in the `satellites` table. |

**Response 200**
```json
{
  "norad_id": 25544,
  "name": "ISS (ZARYA)",
  "kind": "station",
  "description": "International Space Station",
  "tle": {
    "line1": "1 25544U 98067A   26115.13489583  .00012345  00000-0  22345-3 0  9991",
    "line2": "2 25544  51.6428 123.4567 0001234  78.9012 281.1875 15.49876543123456",
    "epoch": "2026-04-25T03:14:15Z",
    "fetched_at": "2026-04-25T06:00:00Z",
    "source": "celestrak"
  },
  "elements": {
    "semi_major_axis_km": 6791.234,
    "eccentricity": 0.0001234,
    "inclination_deg": 51.6428,
    "raan_deg": 123.4567,
    "arg_perigee_deg": 78.9012,
    "mean_anomaly_deg": 281.1875,
    "period_min": 92.68,
    "epoch": "2026-04-25T03:14:15Z"
  }
}
```

**Errors**
- `404 satellite_not_found` — `norad_id` is not in the curated list.
- `503 tle_unavailable` — satellite exists but no TLE has been fetched yet and
  CelesTrak is unreachable (cold start without offline fallback).

**Caching.** `Cache-Control: public, max-age=300`, `ETag` = SHA-256 of the TLE
`(line1, line2)` pair. `If-None-Match` short-circuits to 304.

### `GET /v1/satellites/{norad_id}/tle`

Convenience endpoint that returns just the latest TLE. Useful for debugging
("which TLE am I running?") without pulling the derived elements.

**Response 200**
```json
{
  "norad_id": 25544,
  "line1": "1 25544U 98067A   26115.13489583  .00012345  00000-0  22345-3 0  9991",
  "line2": "2 25544  51.6428 123.4567 0001234  78.9012 281.1875 15.49876543123456",
  "epoch": "2026-04-25T03:14:15Z",
  "fetched_at": "2026-04-25T06:00:00Z",
  "source": "celestrak"
}
```

**Errors** and **caching** same as `/v1/satellites/{norad_id}`.

### `GET /v1/satellites/{norad_id}/trajectory`

Returns a **sampled propagation window** the browser interpolates between for smooth
playback. This is the hottest endpoint; the cache layering described in
[`architecture.md` § Database](./architecture.md#database-postgres) sits behind it.

**Query parameters**

| Name               | Type    | Default | Range          | Description                                      |
|--------------------|---------|---------|----------------|--------------------------------------------------|
| `from`             | string (RFC 3339) | server `now` | now − 30 d to now + 30 d | Window start in UTC.       |
| `duration`         | integer (seconds) | `3600`       | 60 – 86400               | Window length.             |
| `step`             | integer (seconds) | `10`         | 1 – 600                  | Sampling interval.         |
| `include_velocity` | boolean          | `true`       | —                        | Include velocity vectors.  |

The number of samples is exactly `duration / step + 1` (inclusive of both endpoints).
With defaults: `3600 / 10 + 1 = 361`. The browser reads the array length to size
buffers; the server never returns a shorter array than requested.

**Response 200**
```json
{
  "satellite": {
    "norad_id": 25544,
    "name": "ISS (ZARYA)"
  },
  "tle_id": 1234,
  "frame": "eci_j2000",
  "start_at": "2026-04-25T12:00:00Z",
  "duration_s": 3600,
  "step_s": 10,
  "include_velocity": true,
  "hash": "a3f5c8...",
  "computed_at": "2026-04-25T11:59:55Z",
  "cached": true,
  "samples": [
    { "t": 0,    "r_km": [4567.123, 1234.567, 6789.012], "v_km_s": [-3.456, 6.234, 1.122] },
    { "t": 10,   "r_km": [4533.001, 1296.444, 6798.553], "v_km_s": [-3.470, 6.218, 1.080] }
  ]
}
```

**Sample object**

| Field    | Type           | Notes                                                |
|----------|----------------|------------------------------------------------------|
| `t`      | integer        | Seconds since `start_at`. Always a multiple of `step_s`. |
| `r_km`   | `[number,number,number]` | Position in ECI J2000.                     |
| `v_km_s` | `[number,number,number]` | Velocity. **Omitted entirely** when `include_velocity=false`. |

`tle_id` and `hash` are returned so the client can correlate trajectories to the
exact TLE and cache key the worker used. `cached` is `true` when the response was
served from Redis or Postgres without invoking the worker.

**Errors**
- `400 invalid_window` — query parameters violate constraints (e.g. `step > duration`,
  `from` more than 30 days out, non-RFC3339 timestamp). The `details` field lists
  offending parameters.
- `404 satellite_not_found` — unknown `norad_id`.
- `503 tle_unavailable` — no TLE on file (see above).
- `502 propagation_failed` — worker reported an error (Nyx returned an error or the
  TLE failed SGP4 init).
- `504 propagation_timeout` — worker did not respond within the 5 s deadline.

**Caching.** `Cache-Control: public, max-age=300`, `ETag` = `hash`,
`Last-Modified` = `computed_at`. The 5-minute max-age matches the Redis hot-cache
TTL. Browsers and the TanStack Query layer both honour it.

## Error model

All non-2xx responses follow the same shape:

```json
{
  "detail": "Human-readable message",
  "code": "machine_readable_code",
  "details": { "...": "context-specific extras, optional" }
}
```

`detail` and `code` are always present. `details` is optional and only included
when the error has actionable per-field context (typically validation errors).

### Code reference

| HTTP | `code`                    | When it fires                                                |
|------|---------------------------|--------------------------------------------------------------|
| 400  | `invalid_window`          | Trajectory query parameters out of range or malformed.       |
| 400  | `validation_error`        | Generic Pydantic validation failure (other endpoints).       |
| 404  | `satellite_not_found`     | `norad_id` not in the curated list.                          |
| 502  | `propagation_failed`      | Worker returned an error.                                    |
| 503  | `tle_unavailable`         | No TLE on file and CelesTrak is unreachable.                 |
| 503  | `service_unavailable`     | Postgres or Redis is unreachable (also surfaces in `/healthz`). |
| 504  | `propagation_timeout`     | Worker did not publish a result within 5 s.                  |

Clients should branch on `code`, not on `detail` (which may be reworded).

### Example: validation error
```json
{
  "detail": "Trajectory window parameters are invalid",
  "code": "invalid_window",
  "details": {
    "step": "must be between 1 and 600 (received 0)",
    "duration": "must be between 60 and 86400 (received 50)"
  }
}
```

## Things explicitly *not* in v1

- No authentication or rate-limiting headers. Local dev only.
- No `POST`, `PUT`, `PATCH`, or `DELETE`. The whole API is read-only.
- No pagination. Lists are small and fixed.
- No streaming (SSE / WebSocket) endpoints. The trajectory-window pattern is
  enough for v1; SSE is reserved for future "long propagation" endpoints.
- No CORS configuration spec — the dev setup uses Next.js' rewrite proxy, so
  the browser only ever talks to `localhost:3000`.
- No idempotency keys on writes (there are no writes).

## Drift detection

This contract must stay in sync with the FastAPI implementation. Three layers
of defence, increasing in cost:

1. **Hand-maintained endpoint list.** The table at the top of [Endpoints](#endpoints)
   is the canonical list. CI runs a small script (`scripts/check-api-contract.py`)
   that:
   - Generates `apps/api/openapi.json` via the export script.
   - Parses the table out of `docs/api.md` (Markdown table extraction).
   - Asserts the set of `(method, path)` pairs matches exactly.
   - Fails the build with a clear diff if not.
2. **Generated TypeScript.** `packages/types` runs `openapi-typescript` against
   the same `openapi.json` and `apps/web` consumes it. A typo in a field name
   shows up as a TS compile error in `apps/web`, not as a runtime surprise.
3. **Contract tests** (added in M2 of the roadmap, not v1.0): a small `pytest`
   suite that hits a running API and validates each example response in this
   document against the live response with `jsonschema`.

Drift in field-level shape is allowed during development as long as the contract
is updated in the same PR — reviewers should reject PRs that change Pydantic
models without touching this file.

## Open questions

- Should `period_min` round to a fixed number of decimals (e.g. 2) or always
  return full precision? Decision: full precision, formatting is the client's job.
- Do we expose `mean_motion_rev_per_day` alongside `period_min`? Decision deferred
  to PRD; not in v1 unless asked.
- Is `t` in samples better as seconds offset (current decision) or absolute ISO
  timestamps? Decision: seconds offset for size and parsing speed; the client
  reconstructs absolute time from `start_at + t`.
