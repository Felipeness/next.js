//! The execution and persistence backend for `turbo-tasks`.
//!
//! This crate provides [`TurboTasksBackend`], the primary implementation of the `Backend` trait
//! from `turbo-tasks`. It manages task scheduling, dependency tracking, invalidation propagation,
//! and persistent caching of computation results for Turbopack's incremental computation framework.
//!
//! # Getting started
//!
//! Create a backend by combining [`BackendOptions`] with a [`BackingStorage`] implementation:
//!
//! - [`turbo_backing_storage`] — High-performance storage using `turbo-persistence` (recommended)
//! - [`lmdb_backing_storage`] — LMDB-based storage for debugging (requires `lmdb` feature)
//! - [`noop_backing_storage`] — In-memory only, no persistence (for tests)
//! - [`default_backing_storage`] — Picks the best available implementation
//!
//! # Architecture
//!
//! The backend uses snapshot-based persistence: modifications accumulate in memory during normal
//! operation, then are batch-written to disk when the system is idle. Operations are resumable
//! (serializable as enums) and can suspend at safe points during snapshots.
//!
//! Tasks form an aggregation tree for efficient dirty-state propagation. Each task's state is
//! stored in a generated `TaskStorage` struct with fields organized into meta (persisted, rarely
//! changed), data (persisted, frequently changed), and transient (memory-only) categories.

#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(associated_type_defaults)]
#![feature(iter_collect_into)]
#![feature(box_patterns)]

mod backend;
mod backing_storage;
mod data;
mod database;
mod error;
mod kv_backing_storage;
mod utils;

use std::path::Path;

use anyhow::Result;

use crate::database::{noop_kv::NoopKvDb, turbo::TurboKeyValueDatabase};
pub use crate::{
    backend::{BackendOptions, StorageMode, TurboTasksBackend},
    backing_storage::BackingStorage,
    database::{
        db_invalidation, db_invalidation::StartupCacheState, db_versioning::GitVersionInfo,
    },
    kv_backing_storage::KeyValueDatabaseBackingStorage,
};

#[cfg(feature = "lmdb")]
pub type LmdbBackingStorage = KeyValueDatabaseBackingStorage<
    database::read_transaction_cache::ReadTransactionCache<
        database::startup_cache::StartupCacheLayer<
            database::fresh_db_optimization::FreshDbOptimization<
                crate::database::lmdb::LmbdKeyValueDatabase,
            >,
        >,
    >,
>;

/// Creates an [`lmdb`]-based `BackingStorage` to be passed to [`TurboTasksBackend::new`].
///
/// Information about the state of the on-disk cache is returned using [`StartupCacheState`].
///
/// This is backend is slower than [`turbo_backing_storage`], but it's a known-good database that
/// can be used when reproducing user-reported issues to isolate bugs.
///
/// When the `lmdb` cargo feature is enabled, [`default_backing_storage`] will return this value.
#[cfg(feature = "lmdb")]
pub fn lmdb_backing_storage(
    base_path: &Path,
    version_info: &GitVersionInfo,
    is_ci: bool,
) -> Result<(LmdbBackingStorage, StartupCacheState)> {
    use crate::database::{
        fresh_db_optimization::{FreshDbOptimization, is_fresh},
        read_transaction_cache::ReadTransactionCache,
        startup_cache::StartupCacheLayer,
    };

    KeyValueDatabaseBackingStorage::open_versioned_on_disk(
        base_path.to_owned(),
        version_info,
        is_ci,
        |versioned_path| {
            let fresh_db = is_fresh(&versioned_path);
            let database = crate::database::lmdb::LmbdKeyValueDatabase::new(&versioned_path)?;
            let database = FreshDbOptimization::new(database, fresh_db);
            let database =
                StartupCacheLayer::new(database, versioned_path.join("startup.cache"), fresh_db)?;
            Ok(ReadTransactionCache::new(database))
        },
    )
}

pub type TurboBackingStorage = KeyValueDatabaseBackingStorage<TurboKeyValueDatabase>;

/// Creates a `BackingStorage` to be passed to [`TurboTasksBackend::new`].
///
/// Information about the state of the on-disk cache is returned using [`StartupCacheState`].
///
/// This is the fastest most-tested implementation of `BackingStorage`, and is normally returned by
/// [`default_backing_storage`].
pub fn turbo_backing_storage(
    base_path: &Path,
    version_info: &GitVersionInfo,
    is_ci: bool,
    is_short_session: bool,
) -> Result<(TurboBackingStorage, StartupCacheState)> {
    KeyValueDatabaseBackingStorage::open_versioned_on_disk(
        base_path.to_owned(),
        version_info,
        is_ci,
        |path| TurboKeyValueDatabase::new(path, is_ci, is_short_session),
    )
}

pub type NoopBackingStorage = KeyValueDatabaseBackingStorage<NoopKvDb>;

/// Creates an no-op in-memory `BackingStorage` to be passed to [`TurboTasksBackend::new`].
pub fn noop_backing_storage() -> NoopBackingStorage {
    KeyValueDatabaseBackingStorage::new_in_memory(NoopKvDb)
}

#[cfg(feature = "lmdb")]
pub type DefaultBackingStorage = LmdbBackingStorage;

#[cfg(not(feature = "lmdb"))]
pub type DefaultBackingStorage = TurboBackingStorage;

/// Calls [`turbo_backing_storage`] (recommended) or `lmdb_backing_storage`, depending on if the
/// `lmdb` cargo feature is enabled.
pub fn default_backing_storage(
    path: &Path,
    version_info: &GitVersionInfo,
    is_ci: bool,
    is_short_session: bool,
) -> Result<(DefaultBackingStorage, StartupCacheState)> {
    #[cfg(feature = "lmdb")]
    {
        lmdb_backing_storage(path, version_info, is_ci)
    }
    #[cfg(not(feature = "lmdb"))]
    {
        turbo_backing_storage(path, version_info, is_ci, is_short_session)
    }
}
