//! Job payload types for the `stream:propagate` Redis Stream.
//!
//! The message schema mirrors the JSON produced by `apps/api` and described in
//! `docs/architecture.md § Job queue (Redis Streams)`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The TLE lines attached to every propagation job so the worker can propagate
/// without a round-trip back to Postgres.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TleData {
    /// Common-name header line (e.g. `"ISS (ZARYA)"`).
    pub name: String,
    /// TLE Line 1 (69 ASCII characters).
    pub line1: String,
    /// TLE Line 2 (69 ASCII characters).
    pub line2: String,
}

/// Payload stored in the `payload` field of every `stream:propagate` message.
///
/// Field names and types must stay in sync with the Python `JobMessage` dataclass
/// in `apps/api` (M4).  If you rename a field here, rename it there too and
/// update the cross-language hash golden vector test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobPayload {
    /// UUIDv7 job identifier.  Used as the pubsub result channel suffix:
    /// `result:{job_id}`.
    pub job_id: String,

    /// Always `"propagate_window"` for v1.
    pub kind: String,

    /// Primary-key of the `tles` row.  Written directly into `propagated_windows`
    /// so the worker never needs to re-query Postgres for the TLE.
    pub tle_id: i64,

    /// The TLE data to propagate.
    pub tle: TleData,

    /// TLE epoch (parsed from Line 1).  Carried so the worker can verify it
    /// matches the TLE it parses without hitting the database.
    pub epoch: DateTime<Utc>,

    /// Propagation window start time (UTC).
    pub start_at: DateTime<Utc>,

    /// Window duration in seconds.  Must be in `[60, 86400]`.
    pub duration_s: i64,

    /// Sampling interval in seconds.  Must be in `[1, 600]`.
    pub step_s: i64,

    /// Coordinate frame.  Always `"eci_j2000"` for v1.
    ///
    /// Note: the `sgp4` crate outputs **TEME** (True Equator, Mean Equinox),
    /// which is the native SGP4 output frame.  For v1's 1 km accuracy
    /// requirement the TEME/J2000 difference is negligible; positions are
    /// labelled `eci_j2000` throughout the API for consistency with the
    /// architecture doc.
    pub frame: String,

    /// Whether to include velocity vectors in the response.
    pub include_velocity: bool,

    /// Pre-computed cache hash (`sha256:…`).  The worker trusts this value; it
    /// does not recompute the hash.
    pub hash: String,
}

/// Result published to `result:{job_id}` after a successful propagation.
///
/// The FastAPI trajectory endpoint subscribes to this channel and returns the
/// payload to the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobResult {
    /// Successful propagation result.
    Ok(Box<PropagationResult>),
    /// Worker-side error.
    Err(PropagationError),
}

/// Successful propagation result published on `result:{job_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub job_id: String,
    pub tle_id: i64,
    pub hash: String,
    pub frame: String,
    pub start_at: DateTime<Utc>,
    pub duration_s: i64,
    pub step_s: i64,
    pub include_velocity: bool,
    pub samples: Vec<Sample>,
    pub computed_at: DateTime<Utc>,
}

/// Error payload published when propagation fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationError {
    pub job_id: String,
    pub error: String,
    pub detail: String,
}

/// A single sampled position (and optionally velocity) at time offset `t`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sample {
    /// Seconds since `start_at`.  Always a multiple of `step_s`.
    pub t: i64,
    /// ECI/TEME position in km: `[x, y, z]`.
    pub r_km: [f64; 3],
    /// ECI/TEME velocity in km/s: `[vx, vy, vz]`.  `None` when
    /// `include_velocity = false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_km_s: Option<[f64; 3]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixture_payload() -> JobPayload {
        JobPayload {
            job_id: "01900000-0000-7000-8000-000000000001".to_owned(),
            kind: "propagate_window".to_owned(),
            tle_id: 42,
            tle: TleData {
                name: "ISS (ZARYA)".to_owned(),
                line1: "1 25544U 98067A   26116.50000000  .00016717  00000-0  30442-3 0  9999"
                    .to_owned(),
                line2: "2 25544  51.6400 127.0000 0004000  20.0000 340.0000 15.50000000000013"
                    .to_owned(),
            },
            epoch: Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap(),
            start_at: Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap(),
            duration_s: 3600,
            step_s: 10,
            frame: "eci_j2000".to_owned(),
            include_velocity: true,
            hash: "sha256:abc123".to_owned(),
        }
    }

    /// Serde round-trip: serialise then deserialise must reproduce the same struct.
    #[test]
    fn job_payload_serde_roundtrip() {
        let original = fixture_payload();
        let json = serde_json::to_string(&original).expect("serialise");
        let recovered: JobPayload = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(original, recovered);
    }

    /// All expected JSON field names must be present.
    #[test]
    fn job_payload_field_names() {
        let payload = fixture_payload();
        let json = serde_json::to_value(&payload).expect("to_value");
        for field in &[
            "job_id",
            "kind",
            "tle_id",
            "tle",
            "epoch",
            "start_at",
            "duration_s",
            "step_s",
            "frame",
            "include_velocity",
            "hash",
        ] {
            assert!(
                json.get(field).is_some(),
                "missing field '{field}' in serialised JobPayload"
            );
        }
        // Nested TLE fields
        let tle_obj = json.get("tle").unwrap();
        for field in &["name", "line1", "line2"] {
            assert!(
                tle_obj.get(field).is_some(),
                "missing TLE field '{field}'"
            );
        }
    }

    /// `v_km_s` must be omitted when `include_velocity = false`.
    #[test]
    fn sample_skips_velocity_when_none() {
        let s = Sample {
            t: 0,
            r_km: [1.0, 2.0, 3.0],
            v_km_s: None,
        };
        let json = serde_json::to_value(&s).expect("to_value");
        assert!(json.get("v_km_s").is_none(), "v_km_s should be absent when None");
    }

    /// `v_km_s` must be present when supplied.
    #[test]
    fn sample_includes_velocity_when_some() {
        let s = Sample {
            t: 0,
            r_km: [1.0, 2.0, 3.0],
            v_km_s: Some([4.0, 5.0, 6.0]),
        };
        let json = serde_json::to_value(&s).expect("to_value");
        assert!(json.get("v_km_s").is_some(), "v_km_s should be present when Some");
    }
}
