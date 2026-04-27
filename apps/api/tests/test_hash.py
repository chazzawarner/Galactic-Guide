"""Unit tests for the propagation cache-key (hash) computation.

The ``compute_hash`` function must produce output **identical** to the Rust
``worker_lib::hash::compute`` function so that jobs submitted by ``apps/api``
use the same cache key that the worker writes.

The golden vector below was independently computed in Python using:

    import hashlib
    canonical = '1234:2026-04-25T12:00:00+00:00:3600:10:eci_j2000:true'
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    # → 9cdb94ff65c6df3af52c16c1eae7365a558545dd3aedd37bc1567332c07f1f14

The **same** expected value is pinned in ``apps/worker/src/hash.rs``
``#[test] fn golden_hash()``.  Both tests must pass with the same value; a
divergence means the two implementations use different canonical formats.
"""

from __future__ import annotations

import hashlib
from datetime import UTC, datetime


def compute_hash(
    tle_id: int,
    start_at: datetime,
    duration_s: int,
    step_s: int,
    frame: str,
    include_velocity: bool,
) -> str:
    """Compute a deterministic SHA-256 cache key for a propagation window.

    The canonical string format is::

        "{tle_id}:{start_at_rfc3339}:{duration_s}:{step_s}:{frame}:{include_velocity}"

    where ``start_at_rfc3339`` is the ISO 8601 / RFC 3339 representation of the
    UTC timestamp with ``+00:00`` suffix (e.g. ``"2026-04-25T12:00:00+00:00"``),
    and ``include_velocity`` is the Python lowercase string ``"true"`` or
    ``"false"``.

    This format **must** stay in sync with the Rust implementation in
    ``apps/worker/src/hash.rs``.
    """
    start_str = start_at.isoformat()
    iv_str = str(include_velocity).lower()
    canonical = f"{tle_id}:{start_str}:{duration_s}:{step_s}:{frame}:{iv_str}"
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    return f"sha256:{digest}"


# ── Tests ─────────────────────────────────────────────────────────────────────


def test_golden_hash() -> None:
    """Cross-language golden vector: must match Rust hash::tests::golden_hash."""
    start_at = datetime(2026, 4, 25, 12, 0, 0, tzinfo=UTC)
    result = compute_hash(
        tle_id=1234,
        start_at=start_at,
        duration_s=3600,
        step_s=10,
        frame="eci_j2000",
        include_velocity=True,
    )
    assert result == "sha256:9cdb94ff65c6df3af52c16c1eae7365a558545dd3aedd37bc1567332c07f1f14"


def test_false_velocity_different_hash() -> None:
    """include_velocity=False must produce a different hash than True."""
    start_at = datetime(2026, 4, 25, 12, 0, 0, tzinfo=UTC)
    h_true = compute_hash(1234, start_at, 3600, 10, "eci_j2000", True)
    h_false = compute_hash(1234, start_at, 3600, 10, "eci_j2000", False)
    assert h_true != h_false


def test_different_tle_ids_produce_different_hashes() -> None:
    """Different tle_id values must produce different hashes."""
    start_at = datetime(2026, 4, 25, 12, 0, 0, tzinfo=UTC)
    h1 = compute_hash(1, start_at, 3600, 10, "eci_j2000", True)
    h2 = compute_hash(2, start_at, 3600, 10, "eci_j2000", True)
    assert h1 != h2


def test_hash_starts_with_prefix() -> None:
    """Hash must start with the 'sha256:' prefix."""
    start_at = datetime(2026, 4, 25, 12, 0, 0, tzinfo=UTC)
    h = compute_hash(1, start_at, 3600, 10, "eci_j2000", True)
    assert h.startswith("sha256:")


def test_deterministic() -> None:
    """Same inputs must always produce the same hash."""
    start_at = datetime(2026, 4, 25, 12, 0, 0, tzinfo=UTC)
    h1 = compute_hash(42, start_at, 1800, 30, "eci_j2000", False)
    h2 = compute_hash(42, start_at, 1800, 30, "eci_j2000", False)
    assert h1 == h2
