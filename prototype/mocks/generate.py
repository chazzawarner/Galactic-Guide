"""Generate ISS mock trajectory window for the UI prototype.

Runs once. The output JSON is committed; we don't re-run during dev.
Output shape matches docs/api.md: 361 samples * {t, r_km, v_km_s} in ECI J2000.

Usage:
    python3 mocks/generate.py
"""

from __future__ import annotations

import datetime as dt
import json
import pathlib
import sys

from sgp4.api import Satrec, jday

# Pinned ISS TLE (Celestrak, 2024-01-15). Frozen so regenerating gives the same file.
ISS_TLE = (
    "1 25544U 98067A   24015.46569444  .00018525  00000+0  33415-3 0  9994",
    "2 25544  51.6402 174.6291 0003671 117.6936 332.4263 15.49814756434127",
)

WINDOW_DURATION_S = 3600
STEP_S = 10
NORAD_ID = 25544
NAME = "ISS (ZARYA)"


def main() -> int:
    sat = Satrec.twoline2rv(*ISS_TLE)
    start = dt.datetime(2024, 1, 15, 12, 0, 0, tzinfo=dt.timezone.utc)
    samples = []

    for k in range(0, WINDOW_DURATION_S + 1, STEP_S):
        t = start + dt.timedelta(seconds=k)
        jd, fr = jday(t.year, t.month, t.day, t.hour, t.minute, t.second + t.microsecond * 1e-6)
        e, r, v = sat.sgp4(jd, fr)
        if e != 0:
            print(f"sgp4 error {e} at t+{k}s", file=sys.stderr)
            return 1
        samples.append({
            "t": k,
            "r_km": [round(x, 6) for x in r],
            "v_km_s": [round(x, 9) for x in v],
        })

    window = {
        "norad_id": NORAD_ID,
        "name": NAME,
        "frame": "eci_j2000",
        "start_at": start.isoformat().replace("+00:00", "Z"),
        "duration_s": WINDOW_DURATION_S,
        "step_s": STEP_S,
        "include_velocity": True,
        "samples": samples,
    }

    out = pathlib.Path(__file__).parent / f"{NORAD_ID}-trajectory.json"
    out.write_text(json.dumps(window, indent=2) + "\n")
    print(f"wrote {out} — {len(samples)} samples")
    return 0


if __name__ == "__main__":
    sys.exit(main())
