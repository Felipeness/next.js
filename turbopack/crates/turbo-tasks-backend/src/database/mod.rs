//! Database layer providing pluggable key-value storage backends.
//!
//! This module defines the [`KeyValueDatabase`](key_value_database::KeyValueDatabase) trait
//! and provides concrete implementations:
//! - [`turbo`] — High-performance backend using `turbo-persistence` (default)
//! - [`lmdb`] — LMDB backend for debugging/reproduction (feature-gated)
//! - [`noop_kv`] — No-op in-memory backend for tests
//!
//! Supporting modules handle database versioning, cache invalidation, startup caching,
//! and write batch abstractions.

#[cfg(feature = "lmdb")]
mod by_key_space;
pub mod db_invalidation;
pub mod db_versioning;
#[cfg(feature = "lmdb")]
pub mod fresh_db_optimization;
pub mod key_value_database;
#[cfg(feature = "lmdb")]
pub mod lmdb;
pub mod noop_kv;
#[cfg(feature = "lmdb")]
pub mod read_transaction_cache;
#[cfg(feature = "lmdb")]
pub mod startup_cache;
pub mod turbo;
pub mod write_batch;
