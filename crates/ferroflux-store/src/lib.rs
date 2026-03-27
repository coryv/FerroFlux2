use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::str::FromStr;

/// Returns SQLite connection options pre-configured for reliable embedded storage.
///
/// ## Optimizations (especially beneficial for Raspberry Pi / SD Cards)
/// - **WAL Mode**: Reduces write amplification; friendly to flash storage.
/// - **Synchronous Normal**: Reduces fsync frequency while maintaining crash safety.
pub fn sqlite_options_from_url(db_url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    SqliteConnectOptions::from_str(db_url).map(|opts| {
        opts.create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
    })
}
