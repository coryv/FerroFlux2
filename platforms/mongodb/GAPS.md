# Mongodb — Integration Gaps

## Resolved
- **Binary TCP Protocol:** Resolved by dedicated `mongo_query` tool primitive using the official `mongodb` Rust driver.
- **Common Operations:** CRUD operations (Insert, Find, Update, Delete) are implemented as YAML nodes.
- **Transaction Support:** Basic session/transaction support is available.

## Remaining Gaps
- **Complex Aggregations:** UI support for building complex aggregation pipelines is limited.
- **Change Streams:** Not yet supported as a trigger.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
