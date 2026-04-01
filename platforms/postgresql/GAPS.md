# Postgresql — Integration Gaps

## Resolved
- **Binary TCP Protocol:** Resolved by dedicated `sql_query` tool primitive using `sqlx`.
- **Transaction Support:** Resolved with `begin`, `commit`, and `rollback` nodes.
- **Numeric Type Mapping:** Resolved — `sql_query.rs` maps INT/BIGINT → `Value::Number(i64)`, FLOAT/DOUBLE/REAL/NUMERIC/DECIMAL → `Value::Number(f64)`, BOOL/BOOLEAN → `Value::Bool`. No more string coercion.

## Remaining Gaps
- **Advanced Connection Pooling:** No UI to configure max connections or idle timeouts.
- **DECIMAL Precision:** `NUMERIC`/`DECIMAL` columns are mapped to `f64` which can lose precision for high-precision decimals. These should ideally be returned as strings when precision matters.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
