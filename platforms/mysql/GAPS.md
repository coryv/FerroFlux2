# Mysql — Integration Gaps

## Resolved
- **Binary TCP Protocol:** Resolved by dedicated `sql_query` tool primitive using `sqlx`.
- **Transaction Support:** Resolved with `begin`, `commit`, and `rollback` nodes.

## Remaining Gaps
- **Numeric Type Mapping:** Currently, all database columns are mapped to JSON Strings.
  - **Impact:** Low (users must cast in scripts).
  - **Proposed Fix:** Update `sql_query.rs` to map `AnyTypeInfo` to `serde_json::Value` (Number/Bool/etc).
- **Advanced Connection Pooling:** No UI to configure max connections or idle timeouts.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
