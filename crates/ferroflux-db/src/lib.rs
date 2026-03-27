//! # ferroflux-db
//!
//! Database persistence layer for FerroFlux.
//!
//! Absorbs the SQLite connection helper from `ferroflux-store` and provides
//! the `PersistentStore` (workflow/connection/checkpoint storage) and the
//! `DatabaseSecretStore` (encrypted credential resolution) in one focused crate.
//!
//! **Multi-tenancy**: every table includes a `tenant_id` column. All queries
//! MUST include `AND tenant_id = ?` to prevent data leaks across tenants.

pub mod secrets;
pub mod workflow;

// Re-export the SQLite helper (previously in ferroflux-store)
pub use sqlite::sqlite_options_from_url;

/// SQLite connection options pre-configured for reliable embedded storage.
mod sqlite {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
    use std::str::FromStr;

    /// Returns SQLite connection options optimised for WAL mode and low fsync.
    ///
    /// - **WAL mode**: reduces write amplification; friendly to flash/SD storage.
    /// - **Synchronous Normal**: reduces fsync while maintaining crash safety.
    pub fn sqlite_options_from_url(
        db_url: &str,
    ) -> Result<SqliteConnectOptions, sqlx::Error> {
        SqliteConnectOptions::from_str(db_url).map(|opts| {
            opts.create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
        })
    }
}

// Convenience re-exports
pub use secrets::{DatabaseSecretStore, SecretStore};
pub use workflow::PersistentStore;
