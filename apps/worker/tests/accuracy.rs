//! SGP4 propagation accuracy tests.
//!
//! Each test loads a committed golden-vector file produced by the Python `sgp4`
//! package (Vallado algorithm) and verifies that the Rust `sgp4` crate returns
//! positions within **1 km** and velocities within **1 m/s** at each sample.
//!
//! # Acceptance-criteria mapping
//!
//! The 1 km position bound is intentionally tighter than AC #1 (ISS position
//! within 0.1° ≈ 12 km at ISS altitude), so passing this test implies AC #1
//! is satisfied.
//!
//! # Regenerating goldens
//!
//! Run `scripts/regen-goldens.py` and review the diff before committing.
//! CI does **not** regenerate goldens automatically.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A single golden sample: time offset + reference position/velocity.
#[derive(Debug, Deserialize)]
struct GoldenSample {
    t: i64,
    r_km: [f64; 3],
    v_km_s: [f64; 3],
}

/// The structure of each golden JSON file.
#[derive(Debug, Deserialize)]
struct GoldenFile {
    norad_id: u32,
    name: String,
    line1: String,
    line2: String,
    samples: Vec<GoldenSample>,
}

/// Euclidean norm of a 3-vector.
fn norm3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Subtract two 3-vectors element-wise.
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Run accuracy checks for one golden file.
///
/// Loads the TLE, propagates with the Rust `sgp4` crate, and asserts:
/// - Position error  < 1.0 km at every sample.
/// - Velocity error  < 0.001 km/s (= 1 m/s) at every sample.
fn check_golden(golden: &GoldenFile) {
    use sgp4::{Constants, Elements, MinutesSinceEpoch};

    let elements = Elements::from_tle(
        Some(golden.name.clone()),
        golden.line1.as_bytes(),
        golden.line2.as_bytes(),
    )
    .unwrap_or_else(|e| panic!("NORAD {}: TLE parse failed: {e}", golden.norad_id));

    let constants = Constants::from_elements(&elements)
        .unwrap_or_else(|e| panic!("NORAD {}: SGP4 init failed: {e}", golden.norad_id));

    // TLE epoch as a chrono DateTime (for datetime_to_minutes_since_epoch).
    let epoch_datetime = elements.datetime;

    for golden_sample in &golden.samples {
        // Compute the absolute sample time.
        let sample_dt = epoch_datetime + chrono::Duration::seconds(golden_sample.t);

        // Propagation offset in minutes from TLE epoch.
        let sample_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(sample_dt, Utc);
        let minutes = elements
            .datetime_to_minutes_since_epoch(&sample_utc.naive_utc())
            .unwrap_or_else(|e| {
                panic!(
                    "NORAD {}: minutes_since_epoch failed at t={}s: {e}",
                    golden.norad_id, golden_sample.t
                )
            });

        let prediction = constants
            .propagate(MinutesSinceEpoch(minutes.0))
            .unwrap_or_else(|e| {
                panic!(
                    "NORAD {}: SGP4 diverged at t={}s: {e}",
                    golden.norad_id, golden_sample.t
                )
            });

        let pos_err = norm3(sub3(prediction.position, golden_sample.r_km));
        let vel_err = norm3(sub3(prediction.velocity, golden_sample.v_km_s));

        assert!(
            pos_err < 1.0,
            "NORAD {} t={}s: position error {:.4} km ≥ 1.0 km  \
             (rust={:?}, python={:?})",
            golden.norad_id,
            golden_sample.t,
            pos_err,
            prediction.position,
            golden_sample.r_km,
        );

        assert!(
            vel_err < 0.001,
            "NORAD {} t={}s: velocity error {:.6} km/s ≥ 0.001 km/s  \
             (rust={:?}, python={:?})",
            golden.norad_id,
            golden_sample.t,
            vel_err,
            prediction.velocity,
            golden_sample.v_km_s,
        );
    }
}

/// Load a golden file from `tests/golden/sgp4/{norad_id}.json`.
fn load_golden(norad_id: u32) -> GoldenFile {
    let path = format!(
        "{}/tests/golden/sgp4/{norad_id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read golden file '{path}': {e}"));
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("cannot parse golden file '{path}': {e}"))
}

// ── Per-satellite accuracy tests ─────────────────────────────────────────────

/// ISS — AC #1 canonical satellite.  Must be within 1 km at t = 0 (epoch).
#[test]
fn accuracy_iss_25544() {
    check_golden(&load_golden(25544));
}

#[test]
fn accuracy_hubble_20580() {
    check_golden(&load_golden(20580));
}

#[test]
fn accuracy_starlink_44713() {
    check_golden(&load_golden(44713));
}

#[test]
fn accuracy_gps_36585() {
    check_golden(&load_golden(36585));
}

#[test]
fn accuracy_noaa19_33591() {
    check_golden(&load_golden(33591));
}
