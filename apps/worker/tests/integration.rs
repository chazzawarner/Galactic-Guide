//! Integration tests for the propagation worker.
//!
//! These tests require a running Postgres + Redis stack.  They are designed to
//! run via:
//!
//! ```bash
//! docker compose -f docker-compose.yml -f docker-compose.ci.yml \
//!     up -d postgres redis migrate
//! docker compose run --rm worker cargo test --workspace --test integration
//! ```
//!
//! On a developer machine without Docker, set `DATABASE_URL` and `REDIS_URL`
//! manually and run `cargo test --test integration`.
//!
//! Tests that genuinely need an isolated database may opt into a per-test
//! throwaway DB; mark such tests with `#[ignore = "isolated-db"]` and run
//! them under `cargo test --ignored`.
//!
//! # Test structure
//!
//! Each test:
//! 1. Inserts prerequisite rows (satellite + TLE) if not present.
//! 2. Publishes a job to `stream:propagate`.
//! 3. Starts the worker consumer loop in a background task.
//! 4. Subscribes to `result:{job_id}` and waits up to 15 s.
//! 5. Asserts on the row in `propagated_windows` and the pubsub payload.
//! 6. Cleans up test rows (DELETE by hash / test_job_id prefix).

use chrono::{DateTime, TimeZone, Utc};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::timeout;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// ISS TLE from the fallback snapshot.
const ISS_NAME: &str = "ISS (ZARYA)";
const ISS_LINE1: &str =
    "1 25544U 98067A   26116.50000000  .00016717  00000-0  30442-3 0  9999";
const ISS_LINE2: &str =
    "2 25544  51.6400 127.0000 0004000  20.0000 340.0000 15.50000000000013";
const ISS_NORAD: i64 = 25544;

/// Connect to Postgres using `DATABASE_URL`.
async fn pg_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    PgPool::connect(&url)
        .await
        .expect("failed to connect to Postgres")
}

/// Connect to Redis using `REDIS_URL`.
async fn redis_conn() -> MultiplexedConnection {
    let url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379/0".to_owned());
    let client = redis::Client::open(url.as_str()).expect("invalid Redis URL");
    client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis")
}

/// Ensure `satellites` and `tles` rows exist for ISS; return the `tles.id`.
async fn ensure_iss_tle(pool: &PgPool) -> i64 {
    // Upsert satellite row.
    sqlx::query(
        r#"
        INSERT INTO satellites (norad_id, name, kind)
        VALUES ($1, $2, 'station')
        ON CONFLICT (norad_id) DO NOTHING
        "#,
    )
    .bind(ISS_NORAD)
    .bind(ISS_NAME)
    .execute(pool)
    .await
    .expect("upsert satellite");

    // Parse TLE epoch (Jan 1 + day 116.5 of 2026 = April 26 12:00 UTC).
    let epoch: DateTime<Utc> = Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap();

    // Upsert TLE row.
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO tles (norad_id, line1, line2, epoch)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (norad_id, epoch) DO UPDATE SET line1 = EXCLUDED.line1
        RETURNING id
        "#,
    )
    .bind(ISS_NORAD)
    .bind(ISS_LINE1)
    .bind(ISS_LINE2)
    .bind(epoch)
    .fetch_one(pool)
    .await
    .expect("upsert TLE");

    row.0
}

/// Build a serialised `JobPayload` JSON string.
fn build_job_json(job_id: &str, tle_id: i64, hash: &str) -> String {
    let payload = serde_json::json!({
        "job_id": job_id,
        "kind": "propagate_window",
        "tle_id": tle_id,
        "tle": {
            "name": ISS_NAME,
            "line1": ISS_LINE1,
            "line2": ISS_LINE2
        },
        "epoch": "2026-04-26T12:00:00+00:00",
        "start_at": "2026-04-26T12:00:00+00:00",
        "duration_s": 600,
        "step_s": 60,
        "frame": "eci_j2000",
        "include_velocity": true,
        "hash": hash
    });
    serde_json::to_string(&payload).unwrap()
}

/// Delete test artefacts from `propagated_windows` and `stream:propagate`.
async fn cleanup(pool: &PgPool, redis: &mut MultiplexedConnection, hashes: &[&str]) {
    for hash in hashes {
        sqlx::query("DELETE FROM propagated_windows WHERE hash = $1")
            .bind(hash)
            .execute(pool)
            .await
            .ok();
    }
    // Trim the stream to remove test messages (best-effort).
    let _: redis::RedisResult<()> = redis.xtrim("stream:propagate", redis::streams::StreamMaxlen::Equals(0)).await;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Happy-path integration test.
///
/// Publishes a job, starts the worker, waits for the result pubsub message,
/// and asserts that a row with the expected hash appears in `propagated_windows`.
#[tokio::test]
async fn test_happy_path() {
    let pool = pg_pool().await;
    let mut redis = redis_conn().await;

    let tle_id = ensure_iss_tle(&pool).await;

    let job_id = "integration-test-happy-0000-0000-0001";
    let hash = "sha256:integration-test-happy-hash-0000000000000000000000000001";

    // Clean up any leftover rows from a previous run.
    cleanup(&pool, &mut redis, &[hash]).await;

    // Subscribe to the result channel before publishing the job.
    let result_channel = format!("result:{job_id}");
    let mut pubsub = {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379/0".to_owned());
        let client = redis::Client::open(url.as_str()).expect("invalid Redis URL");
        client
            .get_async_pubsub()
            .await
            .expect("failed to get pubsub connection")
    };
    pubsub
        .subscribe(&result_channel)
        .await
        .expect("subscribe");

    // Publish job to stream:propagate.
    let payload_json = build_job_json(job_id, tle_id, hash);
    let _: String = redis
        .xadd("stream:propagate", "*", &[("payload", &payload_json)])
        .await
        .expect("XADD");

    // Start the worker in a background task.
    let pool_clone = pool.clone();
    let redis_for_worker = redis_conn().await;
    tokio::spawn(async move {
        use worker_lib::worker;
        let _ = worker::run(pool_clone, redis_for_worker, "integration-test-worker").await;
    });

    // Wait for the pubsub message (up to 15 s).
    use redis::AsyncCommands as _;
    let msg_result = timeout(Duration::from_secs(15), async {
        let mut stream = pubsub.on_message();
        futures_util::StreamExt::next(&mut stream).await
    })
    .await;

    assert!(
        msg_result.is_ok() && msg_result.unwrap().is_some(),
        "timed out waiting for result on '{result_channel}'"
    );

    // Assert the row exists in propagated_windows.
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM propagated_windows WHERE hash = $1)")
        .bind(hash)
        .fetch_one(&pool)
        .await
        .expect("SELECT EXISTS");
    assert!(exists, "propagated_windows row must exist after job completes");

    // Clean up.
    cleanup(&pool, &mut redis, &[hash]).await;
}

/// Idempotency test: publishing the same job twice must not duplicate the row.
#[tokio::test]
async fn test_idempotency() {
    let pool = pg_pool().await;
    let mut redis = redis_conn().await;

    let tle_id = ensure_iss_tle(&pool).await;
    let job_id = "integration-test-idempotency-0000-0001";
    let hash = "sha256:integration-test-idempotency-hash-00000000000000000000000001";

    cleanup(&pool, &mut redis, &[hash]).await;

    // Publish the same job twice.
    let payload_json = build_job_json(job_id, tle_id, hash);
    for _ in 0..2 {
        let _: String = redis
            .xadd("stream:propagate", "*", &[("payload", &payload_json)])
            .await
            .expect("XADD");
    }

    // Start worker.
    let pool_clone = pool.clone();
    let redis_for_worker = redis_conn().await;
    tokio::spawn(async move {
        use worker_lib::worker;
        let _ = worker::run(pool_clone, redis_for_worker, "integration-test-worker-idem").await;
    });

    // Give the worker time to process both messages.
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Exactly one row must exist.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM propagated_windows WHERE hash = $1",
    )
    .bind(hash)
    .fetch_one(&pool)
    .await
    .expect("COUNT");
    assert_eq!(count, 1, "idempotent: must have exactly one row for hash");

    cleanup(&pool, &mut redis, &[hash]).await;
}
