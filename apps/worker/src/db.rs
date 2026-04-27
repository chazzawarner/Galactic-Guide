//! Database writes for propagated trajectory windows.
//!
//! Only `apps/api` (Alembic) owns schema migrations; the worker writes to
//! `propagated_windows` with an explicit column list so a forgotten migration
//! surfaces as a clear runtime error rather than a silent data mismatch.
//!
//! The `ON CONFLICT (hash) DO NOTHING` clause makes the insert idempotent:
//! re-delivering the same job twice produces exactly one row.

use crate::job::{PropagationResult, Sample};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::PgPool;

/// Insert a propagated window into `propagated_windows`.
///
/// Returns `Ok(())` whether the row was inserted or already existed (conflict
/// is silently ignored).
///
/// # Arguments
/// * `pool` — SQLx connection pool.
/// * `hash` — pre-computed cache key (`"sha256:…"`).
/// * `tle_id` — FK to `tles.id`.
/// * `result` — propagation result to persist.
pub async fn insert_window(pool: &PgPool, result: &PropagationResult) -> Result<()> {
    let samples_json = serde_json::to_value(&result.samples)
        .context("failed to serialise samples to JSON")?;

    sqlx::query(
        r#"
        INSERT INTO propagated_windows
            (hash, tle_id, start_at, duration_s, step_s, frame, include_velocity, samples)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (hash) DO NOTHING
        "#,
    )
    .bind(&result.hash)
    .bind(result.tle_id)
    .bind(result.start_at)
    .bind(result.duration_s as i32)
    .bind(result.step_s as i32)
    .bind(&result.frame)
    .bind(result.include_velocity)
    .bind(samples_json)
    .execute(pool)
    .await
    .context("INSERT INTO propagated_windows failed")?;

    Ok(())
}

/// Fetch samples for a given hash from `propagated_windows`, if the row exists.
///
/// Used by integration tests to verify idempotency without parsing the full
/// result payload.
#[cfg(test)]
pub async fn row_exists(pool: &PgPool, hash: &str) -> Result<bool> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM propagated_windows WHERE hash = $1",
    )
    .bind(hash)
    .fetch_one(pool)
    .await
    .context("COUNT query failed")?;
    Ok(row > 0)
}

/// Build a [`PropagationResult`] from raw parts.
///
/// This is a convenience constructor used by both `worker.rs` and the
/// integration test harness.
pub fn build_result(
    job_id: String,
    tle_id: i64,
    hash: String,
    frame: String,
    start_at: chrono::DateTime<Utc>,
    duration_s: i64,
    step_s: i64,
    include_velocity: bool,
    samples: Vec<Sample>,
) -> PropagationResult {
    PropagationResult {
        job_id,
        tle_id,
        hash,
        frame,
        start_at,
        duration_s,
        step_s,
        include_velocity,
        samples,
        computed_at: Utc::now(),
    }
}
