//! Deterministic cache-key computation for propagated windows.
//!
//! The hash is used as the `hash` column in `propagated_windows` and as the
//! Redis hot-cache key (`cache:result:{hash}`).
//!
//! # Algorithm
//!
//! ```text
//! canonical = "{tle_id}:{start_at_rfc3339}:{duration_s}:{step_s}:{frame}:{include_velocity}"
//! hash      = "sha256:" + hex(SHA-256(canonical.as_bytes()))
//! ```
//!
//! The canonical string must stay identical between this implementation and the
//! Python implementation in `apps/api` (M4).  A committed golden-vector test
//! covers both sides; any change requires updating both implementations in the
//! same PR.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

/// Compute the deterministic cache key for a propagation window.
///
/// # Arguments
/// * `tle_id` — primary key of the `tles` row.
/// * `start_at` — propagation window start (UTC).
/// * `duration_s` — window length in seconds.
/// * `step_s` — sampling interval in seconds.
/// * `frame` — coordinate frame label (e.g. `"eci_j2000"`).
/// * `include_velocity` — whether velocity vectors are included.
///
/// # Returns
/// A `"sha256:{hex}"` string.
pub fn compute(
    tle_id: i64,
    start_at: &DateTime<Utc>,
    duration_s: i64,
    step_s: i64,
    frame: &str,
    include_velocity: bool,
) -> String {
    // RFC 3339 with UTC offset +00:00 (not Z) so Python's datetime.isoformat()
    // produces the same string: `2026-04-25T12:00:00+00:00`.
    let start_str = start_at.to_rfc3339();
    let canonical = format!(
        "{tle_id}:{start_str}:{duration_s}:{step_s}:{frame}:{include_velocity}"
    );
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    format!("sha256:{}", hex::encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Golden vector: the expected SHA-256 was independently computed in Python
    /// using `hashlib.sha256(canonical.encode()).hexdigest()` with the same
    /// canonical format.  Both this test and
    /// `apps/api/tests/unit/test_hash.py` must pass with the same expected
    /// value to prove cross-language agreement.
    #[test]
    fn golden_hash() {
        let start_at = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let result = compute(1234, &start_at, 3600, 10, "eci_j2000", true);
        assert_eq!(
            result,
            "sha256:9cdb94ff65c6df3af52c16c1eae7365a558545dd3aedd37bc1567332c07f1f14"
        );
    }

    /// `include_velocity = false` must produce a different hash.
    #[test]
    fn different_include_velocity_produces_different_hash() {
        let start_at = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let h_true = compute(1234, &start_at, 3600, 10, "eci_j2000", true);
        let h_false = compute(1234, &start_at, 3600, 10, "eci_j2000", false);
        assert_ne!(h_true, h_false);
    }

    /// Different `tle_id` values must produce different hashes.
    #[test]
    fn different_tle_ids_produce_different_hashes() {
        let start_at = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let h1 = compute(1, &start_at, 3600, 10, "eci_j2000", true);
        let h2 = compute(2, &start_at, 3600, 10, "eci_j2000", true);
        assert_ne!(h1, h2);
    }

    /// Hash output must start with the `"sha256:"` prefix.
    #[test]
    fn output_has_sha256_prefix() {
        let start_at = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let h = compute(1, &start_at, 3600, 10, "eci_j2000", true);
        assert!(h.starts_with("sha256:"), "hash must start with 'sha256:'");
    }

    /// Hash function must be deterministic: same inputs → same output.
    #[test]
    fn deterministic() {
        let start_at = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        let h1 = compute(42, &start_at, 1800, 30, "eci_j2000", false);
        let h2 = compute(42, &start_at, 1800, 30, "eci_j2000", false);
        assert_eq!(h1, h2);
    }
}
