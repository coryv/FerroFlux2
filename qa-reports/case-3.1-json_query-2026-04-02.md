# QA Report: Case 3.1 - JSON Query (JMESPath)
**Date:** 2026-04-02
**Status:** ✅ PASS

## Workflow Details
- **Trigger:** `core.trigger.webhook`
- **Nodes:** `core.utils.json` (operation: query), `core.action.log`
- **Description:** Validated JMESPath extraction from nested JSON payload using global workflow configuration.

## Test Results
| Case | Input | Expression | Expected | Result |
|------|-------|------------|----------|--------|
| Simple Path | `{"user": {"profile": {"name": "Alice"}}}` | `user.profile.name` | `"Alice"` | ✅ PASS |
| Array Filter | `{"orders": [{"id": 1, "status": "shipped"}, {"id": 2, "status": "pending"}]}` | `orders[?status == 'pending'].id \| [0]` | `2` | ✅ PASS |

## Engine Verification
- **Global Config**: Verified that `config.` namespace is correctly injected into CEL context.
- **Lazy Materialization**: Verified that nested data refs are resolved only when needed.
- **Port Mapping**: Verified that `result` output from `core.utils.json` is correctly mapped to `transformed` in the node context.

## Evidence
- **Fixture:** `crates/ferroflux-testing/fixtures/json_query.yaml`
- **Test:** `crates/ferroflux-testing/tests/json_query_test.rs`
- **Log Snippet:**
```
INFO Synchronization Gate: All inputs present, executing entity=2v1 trace_id=...
DEBUG Moved ticket source=2v1 target=3v1 port=Some("result") target_handle=Some("Exec")
DEBUG Moved ticket source=2v1 target=3v1 port=Some("result") target_handle=Some("body")
```
