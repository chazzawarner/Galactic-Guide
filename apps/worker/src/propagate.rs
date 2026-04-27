//! SGP4 propagation using the `sgp4` Rust crate (Vallado algorithm).
//!
//! # Coordinate frame note
//!
//! The `sgp4` crate returns positions and velocities in the **TEME** (True
//! Equator, Mean Equinox) frame, which is the native frame of the SGP4
//! algorithm.  The architecture doc labels the output as `"eci_j2000"` for
//! API consistency; at the v1 accuracy requirement of 1 km, the TEME/J2000
//! difference (a small rotation due to precession of the equinoxes, on the
//! order of tens of metres for LEO satellites) is negligible.
//!
//! # Axis convention (Nyx/three.js interop)
//!
//! SGP4 / TEME uses the standard right-handed ECI convention:
//! - X points toward the vernal equinox.
//! - Y completes the right-handed system.
//! - Z points toward the celestial north pole.
//!
//! three.js uses Y-up.  When rendering, **three.js Y ← TEME Z**.
//! This mapping is applied once in `apps/web/lib/gmst.ts`; it is NOT applied
//! here.  The worker always returns raw TEME vectors.

use crate::job::Sample;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sgp4::{Constants, Elements, MinutesSinceEpoch};

/// Propagate a TLE over a sampled window.
///
/// # Arguments
/// * `name` — satellite name (used as the TLE header line).
/// * `line1` / `line2` — TLE lines.
/// * `start_at` — window start time (UTC).
/// * `duration_s` — window length in seconds (`60 ≤ d ≤ 86400`).
/// * `step_s` — sampling interval in seconds (`1 ≤ s ≤ 600`, `s ≤ d`).
/// * `include_velocity` — whether to include velocity in each [`Sample`].
///
/// # Returns
/// A `Vec<Sample>` with exactly `duration_s / step_s + 1` entries
/// (inclusive of both endpoints), each at `t = k * step_s` seconds from
/// `start_at`.
///
/// # Errors
/// Returns an error if the TLE cannot be parsed or if SGP4 diverges for any
/// sample.
pub fn propagate_window(
    name: &str,
    line1: &str,
    line2: &str,
    start_at: &DateTime<Utc>,
    duration_s: i64,
    step_s: i64,
    include_velocity: bool,
) -> Result<Vec<Sample>> {
    // Parse TLE into sgp4 Elements.
    let elements = Elements::from_tle(
        Some(name.to_owned()),
        line1.as_bytes(),
        line2.as_bytes(),
    )
    .context("failed to parse TLE")?;

    // Initialise SGP4 constants (Brouwer mean elements).
    let constants = Constants::from_elements(&elements).context("failed to initialise SGP4")?;

    // Number of samples: inclusive on both endpoints.
    let n_samples = (duration_s / step_s) + 1;
    let mut samples = Vec::with_capacity(n_samples as usize);

    for k in 0..n_samples {
        let t_secs = k * step_s;
        // MinutesSinceEpoch is minutes from the TLE epoch.
        let sample_time = *start_at + chrono::Duration::seconds(t_secs);
        let minutes = elements
            .datetime_to_minutes_since_epoch(&sample_time.naive_utc())
            .with_context(|| format!("datetime_to_minutes_since_epoch failed at t={t_secs}s"))?;
        let prediction = constants
            .propagate(MinutesSinceEpoch(minutes.0))
            .with_context(|| format!("SGP4 propagation diverged at t={t_secs}s"))?;

        samples.push(Sample {
            t: t_secs,
            r_km: prediction.position,
            v_km_s: if include_velocity {
                Some(prediction.velocity)
            } else {
                None
            },
        });
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ISS TLE from the fallback snapshot (apps/api/data/celestrak-fallback.json).
    const ISS_LINE1: &str =
        "1 25544U 98067A   26116.50000000  .00016717  00000-0  30442-3 0  9999";
    const ISS_LINE2: &str =
        "2 25544  51.6400 127.0000 0004000  20.0000 340.0000 15.50000000000013";

    fn iss_epoch() -> DateTime<Utc> {
        // TLE epoch: 2026 day 116.5 = April 26 12:00:00 UTC
        Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap()
    }

    /// With default ISS TLE, `duration=3600, step=10` must yield exactly 361 samples.
    #[test]
    fn sample_count_is_inclusive() {
        let start = iss_epoch();
        let samples =
            propagate_window("ISS", ISS_LINE1, ISS_LINE2, &start, 3600, 10, true).unwrap();
        assert_eq!(samples.len(), 361, "expected 361 samples (3600/10 + 1)");
    }

    /// `t` values must start at 0 and be monotonically increasing multiples of `step_s`.
    #[test]
    fn t_values_monotonic_and_aligned() {
        let start = iss_epoch();
        let samples =
            propagate_window("ISS", ISS_LINE1, ISS_LINE2, &start, 3600, 10, true).unwrap();
        assert_eq!(samples[0].t, 0, "first t must be 0");
        assert_eq!(*samples.last().unwrap(), samples[360]);
        assert_eq!(samples.last().unwrap().t, 3600, "last t must equal duration_s");
        for (i, s) in samples.iter().enumerate() {
            assert_eq!(s.t, i as i64 * 10, "t[{i}] must be {}", i * 10);
        }
    }

    /// `duration=60, step=60` must yield exactly 2 samples (both endpoints).
    #[test]
    fn minimum_window_two_samples() {
        let start = iss_epoch();
        let samples =
            propagate_window("ISS", ISS_LINE1, ISS_LINE2, &start, 60, 60, true).unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].t, 0);
        assert_eq!(samples[1].t, 60);
    }

    /// `v_km_s` must be `None` for all samples when `include_velocity = false`.
    #[test]
    fn velocity_absent_when_not_requested() {
        let start = iss_epoch();
        let samples =
            propagate_window("ISS", ISS_LINE1, ISS_LINE2, &start, 600, 60, false).unwrap();
        for s in &samples {
            assert!(s.v_km_s.is_none(), "v_km_s should be None when include_velocity=false");
        }
    }

    /// `v_km_s` must be `Some(...)` for all samples when `include_velocity = true`.
    #[test]
    fn velocity_present_when_requested() {
        let start = iss_epoch();
        let samples =
            propagate_window("ISS", ISS_LINE1, ISS_LINE2, &start, 600, 60, true).unwrap();
        for s in &samples {
            assert!(s.v_km_s.is_some(), "v_km_s should be Some when include_velocity=true");
        }
    }

    /// Invalid TLE must return an error, not panic.
    #[test]
    fn invalid_tle_returns_error() {
        let start = iss_epoch();
        let result =
            propagate_window("BAD", "not a valid line1", "not a valid line2", &start, 60, 10, true);
        assert!(result.is_err(), "invalid TLE must return Err");
    }
}

// ── Proptest property-based tests ────────────────────────────────────────────

/// Property-based tests that verify invariants hold for arbitrary valid inputs.
///
/// Set `PROPTEST_CASES=64` in the environment (the default in docker-compose.yml).
/// The seed is pinned in `.proptest-regressions/propagate.txt` so failures are
/// reproducible.
#[cfg(test)]
mod proptests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    const ISS_LINE1: &str =
        "1 25544U 98067A   26116.50000000  .00016717  00000-0  30442-3 0  9999";
    const ISS_LINE2: &str =
        "2 25544  51.6400 127.0000 0004000  20.0000 340.0000 15.50000000000013";

    fn iss_epoch() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap()
    }

    proptest! {
        /// For any valid (duration_s, step_s) pair with step_s ≤ duration_s, the
        /// sample count is always `duration_s / step_s + 1`.
        #[test]
        fn sample_count_formula(
            // step_s ∈ [1, 600], duration_s ∈ [step_s, 86400]
            step_s in 1i64..=600i64,
            duration_multiplier in 1i64..=144i64,
        ) {
            // Clamp duration so it's >= step_s and ≤ 86400.
            let duration_s = (step_s * duration_multiplier).min(86400);
            let start = iss_epoch();
            let samples = propagate_window(
                "ISS",
                ISS_LINE1,
                ISS_LINE2,
                &start,
                duration_s,
                step_s,
                false,
            )
            .expect("propagation must succeed for valid inputs");

            let expected_count = duration_s / step_s + 1;
            prop_assert_eq!(
                samples.len() as i64,
                expected_count,
                "expected {} samples for duration={}, step={}",
                expected_count,
                duration_s,
                step_s,
            );
        }

        /// First sample `t` is always 0; last sample `t` is always `duration_s`.
        #[test]
        fn first_and_last_t(
            step_s in 1i64..=600i64,
            duration_multiplier in 1i64..=10i64,
        ) {
            let duration_s = (step_s * duration_multiplier).min(86400);
            let start = iss_epoch();
            let samples = propagate_window(
                "ISS",
                ISS_LINE1,
                ISS_LINE2,
                &start,
                duration_s,
                step_s,
                false,
            )
            .expect("propagation must succeed");
            prop_assert_eq!(samples.first().unwrap().t, 0);
            prop_assert_eq!(samples.last().unwrap().t, duration_s);
        }
    }
}
