//! Redis Streams consumer loop.
//!
//! Reads propagation jobs from `stream:propagate` using `XREADGROUP`, calls
//! the SGP4 propagator, writes the result to Postgres, publishes on the result
//! channel, and acknowledges the message.
//!
//! # Error handling
//!
//! - **Deserialise failure** — the message is ACKed and an error result is
//!   published so the FastAPI waiter does not time out.
//! - **Propagation failure** — same treatment: ACK + error result.
//! - **DB failure** — logged; error result published; message is still ACKed.
//! - **Publish failure** — logged; the API timeout (`propagation_timeout`) will
//!   fire.  The DB row (if written) remains for future cache hits.
//!
//! In all failure cases the message is ACKed to prevent an unbounded pending
//! list.

use crate::db;
use crate::job::{JobPayload, JobResult, PropagationError};
use crate::propagate;
use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sqlx::PgPool;
use tracing::{error, info, warn};

const STREAM_KEY: &str = "stream:propagate";
const GROUP_NAME: &str = "workers";
/// Block up to 5 seconds waiting for new messages before looping.
const BLOCK_MS: usize = 5_000;
/// Max messages to consume per XREADGROUP call.
const COUNT: usize = 10;

/// Run the consumer loop indefinitely.
///
/// # Arguments
/// * `pool` — SQLx connection pool for Postgres writes.
/// * `redis` — multiplexed Redis connection.
/// * `worker_name` — unique name for this instance within the consumer group.
pub async fn run(
    pool: PgPool,
    mut redis: MultiplexedConnection,
    worker_name: &str,
) -> Result<()> {
    info!(worker_name, "worker started, consuming from stream '{STREAM_KEY}'");

    loop {
        let messages: Vec<redis::streams::StreamReadReply> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(GROUP_NAME)
            .arg(worker_name)
            .arg("COUNT")
            .arg(COUNT)
            .arg("BLOCK")
            .arg(BLOCK_MS)
            .arg("STREAMS")
            .arg(STREAM_KEY)
            .arg(">")
            .query_async(&mut redis)
            .await
            .unwrap_or_else(|e| {
                warn!("XREADGROUP failed: {e}");
                vec![]
            });

        for stream_reply in messages {
            for stream_id_reply in stream_reply.keys {
                for entry in stream_id_reply.ids {
                    let msg_id = entry.id.clone();
                    process_message(&pool, &mut redis, worker_name, &msg_id, &entry).await;
                }
            }
        }
    }
}

/// Process a single stream entry.
async fn process_message(
    pool: &PgPool,
    redis: &mut MultiplexedConnection,
    worker_name: &str,
    msg_id: &str,
    entry: &redis::streams::StreamId,
) {
    // ── 1. Deserialise ────────────────────────────────────────────────────────
    let payload_str: String = match entry.get("payload") {
        Some(v) => v,
        None => {
            warn!(msg_id, "message missing 'payload' field — ACKing");
            ack(redis, msg_id).await;
            return;
        }
    };

    let payload: JobPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(e) => {
            warn!(msg_id, "failed to deserialise job payload: {e}");
            // We can't publish a typed error without a job_id; log and ACK.
            ack(redis, msg_id).await;
            return;
        }
    };

    let job_id = payload.job_id.clone();
    info!(job_id, msg_id, "processing propagation job");

    // ── 2. Propagate ─────────────────────────────────────────────────────────
    let samples = match propagate::propagate_window(
        &payload.tle.name,
        &payload.tle.line1,
        &payload.tle.line2,
        &payload.start_at,
        payload.duration_s,
        payload.step_s,
        payload.include_velocity,
    ) {
        Ok(s) => s,
        Err(e) => {
            error!(job_id, "SGP4 propagation failed: {e:#}");
            publish_error(redis, &job_id, "propagation_failed", &format!("{e:#}")).await;
            ack(redis, msg_id).await;
            return;
        }
    };

    // ── 3. Build result ───────────────────────────────────────────────────────
    let result = db::build_result(
        job_id.clone(),
        payload.tle_id,
        payload.hash.clone(),
        payload.frame.clone(),
        payload.start_at,
        payload.duration_s,
        payload.step_s,
        payload.include_velocity,
        samples,
    );

    // ── 4. Persist ────────────────────────────────────────────────────────────
    if let Err(e) = db::insert_window(pool, &result).await {
        error!(job_id, "DB insert failed: {e:#}");
        // Publish an error so the API waiter doesn't time out; ACK anyway.
        publish_error(redis, &job_id, "propagation_failed", &format!("DB error: {e:#}")).await;
        ack(redis, msg_id).await;
        return;
    }

    // ── 5. Publish result ────────────────────────────────────────────────────
    let result_json = match serde_json::to_string(&JobResult::Ok(Box::new(result))) {
        Ok(j) => j,
        Err(e) => {
            error!(job_id, "failed to serialise result: {e}");
            publish_error(redis, &job_id, "propagation_failed", "serialisation error").await;
            ack(redis, msg_id).await;
            return;
        }
    };
    let channel = format!("result:{job_id}");
    if let Err(e) = redis.publish::<_, _, ()>(&channel, &result_json).await {
        warn!(job_id, "PUBLISH on '{channel}' failed: {e}");
    } else {
        info!(job_id, "published result on '{channel}'");
    }

    // ── 6. Acknowledge ───────────────────────────────────────────────────────
    ack(redis, msg_id).await;
    info!(job_id, msg_id, worker_name, "job complete");
}

/// Send `XACK stream:propagate workers {msg_id}`.
async fn ack(redis: &mut MultiplexedConnection, msg_id: &str) {
    if let Err(e) = redis.xack::<_, _, _, ()>(STREAM_KEY, GROUP_NAME, &[msg_id]).await {
        warn!(msg_id, "XACK failed: {e}");
    }
}

/// Publish a typed error payload to `result:{job_id}`.
async fn publish_error(
    redis: &mut MultiplexedConnection,
    job_id: &str,
    error_code: &str,
    detail: &str,
) {
    let err = PropagationError {
        job_id: job_id.to_owned(),
        error: error_code.to_owned(),
        detail: detail.to_owned(),
    };
    if let Ok(json) = serde_json::to_string(&JobResult::Err(err)) {
        let channel = format!("result:{job_id}");
        if let Err(e) = redis.publish::<_, _, ()>(&channel, &json).await {
            warn!(job_id, "PUBLISH error result failed: {e}");
        }
    }
}
