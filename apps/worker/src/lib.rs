//! Library crate for the Galactic Guide propagation worker.
//!
//! Exposes the internal modules so that integration tests and other crates can
//! import and call worker logic directly without spawning a subprocess.

pub mod config;
pub mod db;
pub mod hash;
pub mod job;
pub mod propagate;
pub mod worker;
