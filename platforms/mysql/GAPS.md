# Mysql — Integration Gaps

## Resolved
- **Binary TCP Protocol:** Resolved by dedicated `sql_query` tool primitive using `sqlx`.
- **Transaction Support:** Resolved with `begin`, `commit`, and `rollback` nodes.
- **Numeric Type Mapping:** Resolved — `sql_query.rs` maps INT/BIGINT → `Value::Number(i64)`, FLOAT/DOUBLE/REAL/NUMERIC/DECIMAL → `Value::Number(f64)`, BOOL/BOOLEAN → `Value::Bool`.

## Remaining Gaps
- **Advanced Connection Pooling:** No UI to configure max connections or idle timeouts.
- **DECIMAL Precision:** `DECIMAL` columns mapped to `f64` may lose precision for financial calculations. Return as string when precision is critical.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
