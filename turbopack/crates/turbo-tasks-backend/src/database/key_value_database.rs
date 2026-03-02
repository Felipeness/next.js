//! Abstract key-value database trait.
//!
//! [`KeyValueDatabase`] defines the interface for low-level storage backends. Implementations
//! must support read transactions, key-space-scoped reads, and write batches (serial or
//! concurrent). The trait is used by [`KeyValueDatabaseBackingStorage`](crate::kv_backing_storage)
//! to bridge to the [`BackingStorage`](crate::BackingStorage) trait.

use anyhow::Result;

use crate::database::write_batch::{
    ConcurrentWriteBatch, SerialWriteBatch, UnimplementedWriteBatch, WriteBatch,
};

/// Logical key namespaces within the database, stored as separate families/tables.
#[derive(Debug, Clone, Copy)]
pub enum KeySpace {
    Infra = 0,
    TaskMeta = 1,
    TaskData = 2,
    TaskCache = 3,
}

/// A low-level key-value database supporting transactional reads and batched writes.
///
/// Implementations provide the storage layer for [`BackingStorage`](crate::BackingStorage).
/// The database operates on four [`KeySpace`]s (Infra, TaskMeta, TaskData, TaskCache).
pub trait KeyValueDatabase {
    type ReadTransaction<'l>
    where
        Self: 'l;

    fn begin_read_transaction(&self) -> Result<Self::ReadTransaction<'_>>;

    fn is_empty(&self) -> bool {
        false
    }

    type ValueBuffer<'l>: std::borrow::Borrow<[u8]>
    where
        Self: 'l;

    fn get<'l, 'db: 'l>(
        &'l self,
        transaction: &'l Self::ReadTransaction<'db>,
        key_space: KeySpace,
        key: &[u8],
    ) -> Result<Option<Self::ValueBuffer<'l>>>;

    fn batch_get<'l, 'db: 'l>(
        &'l self,
        transaction: &'l Self::ReadTransaction<'db>,
        key_space: KeySpace,
        keys: &[&[u8]],
    ) -> Result<Vec<Option<Self::ValueBuffer<'l>>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            let value = self.get(transaction, key_space, key)?;
            results.push(value);
        }
        Ok(results)
    }

    type SerialWriteBatch<'l>: SerialWriteBatch<'l>
        = UnimplementedWriteBatch
    where
        Self: 'l;
    type ConcurrentWriteBatch<'l>: ConcurrentWriteBatch<'l>
        = UnimplementedWriteBatch
    where
        Self: 'l;

    fn write_batch(
        &self,
    ) -> Result<WriteBatch<'_, Self::SerialWriteBatch<'_>, Self::ConcurrentWriteBatch<'_>>>;

    /// Called when the database has been invalidated via
    /// [`crate::backing_storage::BackingStorage::invalidate`]
    ///
    /// This typically means that we'll restart the process or `turbo-tasks` soon with a fresh
    /// database. If this happens, there's no point in writing anything else to disk, or flushing
    /// during [`KeyValueDatabase::shutdown`].
    ///
    /// This is a best-effort optimization hint, and the database may choose to ignore this and
    /// continue file writes. This happens after the database is invalidated, so it is valid for
    /// this to leave the database in a half-updated and corrupted state.
    fn prevent_writes(&self) {
        // this is an optional performance hint to the database
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
