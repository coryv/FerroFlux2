# FerroFlux QA Report

**Date:** 2026-04-05
**Workflow:** Webhook -> Date Extract -> Date Diff -> HTTP Verify
**Nodes tested:** `core.trigger.webhook`, `core.utils.json`, `core.utils.date`, `core.action.http`
**Test file:** `crates/ferroflux-testing/tests/case_3_7_date_parse_diff_test.rs`

---

## Result: ❌ FAIL

---

## Workflow Under Test

| Step | Node Type | HTTP Call(s) |
|------|-----------|--------------|
| Trigger | `core.trigger.webhook` | — |
| 1-start | `core.utils.json` | — |
| 1-end | `core.utils.json` | — |
| 2 | `core.utils.date` (diff) | — |
| 3 | `core.action.http` | — |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| Extraction nodes succeed | ✅ | Tickets moved to diff_node |
| Date Diff executes | ❌ | Fails with `premature end of input` |
| HTTP Verify called | ❌ | Pipeline failed before node |

## Engine Contract Violations

The engine's `pipeline_execution_system` passed the `Synchronization Gate`, confirming all inputs (`Exec`, `date1`, `date2`) were present. However, during execution of the `core.utils.date` node, the CEL resolver failed with `No such key: date1`, causing it to fall back to the literal string expression `"inputs.date1"`. This indicates a failure in the engine's data materialization layer where arriving tickets are not correctly mapped into the `inputs` object passed to the CEL interpreter for multi-input nodes.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | FAIL |
| Cargo test duration | ~1.81s |
| HTTP calls expected | 1 |
| HTTP calls received | 0 |
| Assertions | 1/3 passed |

## Raw Test Output

```
2026-04-05T15:35:44.168651Z  INFO ferroflux_core::systems::pipeline: Synchronization Gate: All inputs present, executing entity=3v1 trace_id=4f79ce86-a150-44f8-b418-6e5a9a05f175
2026-04-05T15:35:44.170318Z DEBUG ferroflux_core::systems::pipeline::resolution: CEL execution failed during resolution, using literal string expression="inputs.date1" error=No such key: date1
2026-04-05T15:35:44.170404Z DEBUG ferroflux_core::systems::pipeline::resolution: CEL execution failed during resolution, using literal string expression="inputs.date2" error=No such key: date2
2026-04-05T15:35:44.170759Z ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: input contains invalid characters trace_id=4f79ce86-a150-44f8-b418-6e5a9a05f175
...
thread 'test_case_3_7_date_parse_diff' panicked at crates/ferroflux-testing/tests/case_3_7_date_parse_diff_test.rs:78:5:
expected at least 1 HTTP call to /verify-diff
```
