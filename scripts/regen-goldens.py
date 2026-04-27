#!/usr/bin/env python3
"""Generate golden SGP4 vectors for the five curated satellites.

Reads TLEs from ``apps/api/data/celestrak-fallback.json`` and propagates at
``t ∈ {0, 60, 600, 3600}`` seconds using the Python ``sgp4`` package
(Vallado SGP4, same algorithm as the Rust ``sgp4`` crate).

Writes ``apps/worker/tests/golden/sgp4/{norad_id}.json``.

Usage::

    # From the repository root:
    python scripts/regen-goldens.py

    # Or from anywhere with explicit paths:
    python scripts/regen-goldens.py \\
        --fallback apps/api/data/celestrak-fallback.json \\
        --output   apps/worker/tests/golden/sgp4

The generated files are committed.  **Any diff requires a reviewed PR.**
CI does not regenerate goldens automatically.

Requirements::

    pip install sgp4

Cross-language agreement
------------------------
The Rust ``sgp4`` crate and this Python script use the same Vallado SGP4
algorithm, so numerical results should agree to within floating-point
rounding (≪ 1 mm).  The Rust accuracy tests in
``apps/worker/tests/accuracy.rs`` assert position error < 1 km and velocity
error < 1 m/s against these files.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from sgp4.api import Satrec
    from sgp4.conveniences import jday
except ImportError:
    print(
        "ERROR: 'sgp4' package not found.  Install it with: pip install sgp4",
        file=sys.stderr,
    )
    sys.exit(1)

# Propagation time offsets in seconds.
OFFSETS_S: list[int] = [0, 60, 600, 3600]


def parse_tle_epoch(line1: str) -> tuple[int, float]:
    """Return ``(year, day_frac)`` from TLE Line 1 epoch field (chars 18–31).

    ``year`` is the four-digit year; ``day_frac`` is the fractional day of year
    (1-indexed, inclusive of fractional seconds).
    """
    epoch_str = line1[18:32]
    year_2d = int(epoch_str[:2])
    year = 2000 + year_2d if year_2d < 57 else 1900 + year_2d  # noqa: PLR2004
    day_frac = float(epoch_str[2:])
    return year, day_frac


def generate_golden(
    norad_id: int,
    name: str,
    line1: str,
    line2: str,
    offsets: list[int],
) -> dict:  # type: ignore[type-arg]
    """Propagate a TLE at each offset and return the golden dict."""
    sat = Satrec.twoline2rv(line1, line2)

    # Derive the TLE epoch Julian date.
    year, day_frac = parse_tle_epoch(line1)
    day_int = int(day_frac)
    frac = day_frac - day_int

    # Julian date for Jan 1 of the TLE year at 00:00:00 UTC.
    jd_jan1, fr_jan1 = jday(year, 1, 1, 0, 0, 0.0)
    # Add (day - 1) whole days plus the fractional day to reach the epoch.
    jd_epoch = jd_jan1 + (day_int - 1) + frac
    fr_epoch = fr_jan1  # fr_jan1 is 0 since we passed seconds=0

    samples = []
    for t_s in offsets:
        # Add t_s seconds to the epoch Julian date.
        delta_jd = t_s / 86400.0
        error_code, r, v = sat.sgp4(jd_epoch + delta_jd, fr_epoch)
        if error_code != 0:
            raise RuntimeError(
                f"sgp4 returned error {error_code} for NORAD {norad_id} at t={t_s}s"
            )
        samples.append(
            {
                "t": t_s,
                "r_km": list(r),
                "v_km_s": list(v),
            }
        )

    return {
        "norad_id": norad_id,
        "name": name,
        "line1": line1,
        "line2": line2,
        "samples": samples,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    repo_root = Path(__file__).parent.parent
    parser.add_argument(
        "--fallback",
        type=Path,
        default=repo_root / "apps" / "api" / "data" / "celestrak-fallback.json",
        help="Path to celestrak-fallback.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=repo_root / "apps" / "worker" / "tests" / "golden" / "sgp4",
        help="Output directory for golden JSON files",
    )
    args = parser.parse_args()

    fallback_path: Path = args.fallback
    output_dir: Path = args.output

    if not fallback_path.exists():
        print(f"ERROR: fallback file not found: {fallback_path}", file=sys.stderr)
        sys.exit(1)

    with fallback_path.open() as fh:
        satellites: list[dict] = json.load(fh)  # type: ignore[type-arg]

    output_dir.mkdir(parents=True, exist_ok=True)

    for sat in satellites:
        norad_id: int = sat["norad_id"]
        name: str = sat["name"]
        line1: str = sat["line1"]
        line2: str = sat["line2"]

        golden = generate_golden(norad_id, name, line1, line2, OFFSETS_S)
        out_path = output_dir / f"{norad_id}.json"
        with out_path.open("w") as fh:
            json.dump(golden, fh, indent=2)
        print(f"  wrote {out_path.relative_to(repo_root)} ({name})")

    print(f"\nGenerated {len(satellites)} golden files in {output_dir.relative_to(repo_root)}")
    print("Review the diff before committing — any change requires a reviewed PR.")


if __name__ == "__main__":
    main()
