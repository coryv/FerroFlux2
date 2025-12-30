#[cfg(feature = "clickhouse")]
pub mod clickhouse;
#[cfg(feature = "duckdb")]
pub mod duckdb;

#[cfg(feature = "clickhouse")]
pub use clickhouse::ClickHouseStore;
#[cfg(feature = "duckdb")]
pub use duckdb::DuckDbStore;
