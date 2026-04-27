//! Galactic Guide propagation worker.
//!
//! Entry point for the Rust SGP4 worker process.
//!
//! # Startup sequence
//! 1. Read configuration from environment variables ([`worker_lib::config::Config`]).
//! 2. Connect to Postgres via [`sqlx`] connection pool.
//! 3. Connect to Redis (multiplexed async connection).
//! 4. Ensure the `stream:propagate` consumer group exists (idempotent).
//! 5. Run the consumer loop indefinitely ([`worker_lib::worker::run`]).

use anyhow::{Context, Result};
use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use worker_lib::{config, worker};

const STREAM_KEY: &str = "stream:propagate";
const GROUP_NAME: &str = "workers";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = config::Config::from_env().context("failed to load configuration")?;
    info!(worker_name = %cfg.worker_name, "starting Galactic Guide propagation worker");

    // ── Postgres ──────────────────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .context("failed to connect to Postgres")?;
    info!("connected to Postgres");

    // ── Redis ─────────────────────────────────────────────────────────────────
    let redis_client =
        redis::Client::open(cfg.redis_url.as_str()).context("invalid Redis URL")?;
    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .context("failed to connect to Redis")?;
    info!("connected to Redis");

    // Ensure the consumer group exists.  BUSYGROUP means it already exists — OK.
    let create_result: redis::RedisResult<()> = redis_conn
        .xgroup_create_mkstream(STREAM_KEY, GROUP_NAME, "$")
        .await;
    match create_result {
        Ok(_) => info!("created consumer group '{GROUP_NAME}' on '{STREAM_KEY}'"),
        Err(e) if e.to_string().contains("BUSYGROUP") => {
            info!("consumer group '{GROUP_NAME}' already exists")
        }
        Err(e) => {
            tracing::warn!("could not ensure consumer group exists: {e}");
        }
    }

    // ── Consumer loop ─────────────────────────────────────────────────────────
    worker::run(pool, redis_conn, &cfg.worker_name).await
}

