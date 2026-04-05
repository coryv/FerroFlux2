# FerroFlux QA Report

**Date:** 2026-04-05
**Workflow:** Webhook -> Crypto Hash -> Crypto UUID -> HTTP Verify
**Nodes tested:** `core.trigger.webhook`, `core.utils.crypto`, `core.action.http`
**Test file:** `crates/ferroflux-testing/tests/case_3_8_crypto_hash_uuid_test.rs`

---

## Result: ❌ FAIL

---

## Workflow Under Test

| Step | Node Type | HTTP Call(s) |
|------|-----------|--------------|
| Trigger | `core.trigger.webhook` | — |
| 1 | `core.utils.crypto` (hash) | — |
| 2 | `core.utils.crypto` (uuid) | — |
| 3 | `core.action.http` | — |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| Hash node executes | ❌ | Fails with `Missing 'input'` |
| UUID node executes | ❌ | Pipeline failed before node |
| HTTP Verify called | ❌ | Pipeline failed before node |

## Engine Contract Violations

The `core.utils.crypto` (hash) node failed with `Missing 'input'`, similar to the failure in 3.7. The payload passed to the `hasher` node was the `body` of the webhook triggered with `{"input": "hello world"}`. However, the node's CEL resolution (using `has(inputs.input) ? inputs.input : ...`) failed to correctly propagate the `input` string to the tool. This confirms a systemic issue in the FerroFlux engine's `inputs` materialization for utility nodes.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | FAIL |
| Cargo test duration | ~1.90s |
| HTTP calls expected | 1 |
| HTTP calls received | 0 |
| Assertions | 0/3 passed |

## Raw Test Output

```
2026-04-05T15:38:39.343844Z  INFO ferroflux_core::systems::pipeline: Synchronization Gate: All inputs present, executing entity=1v1 trace_id=f9b58869-c78e-4b2b-916f-14ece4356ab4
2026-04-05T15:38:39.349275Z ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: Missing 'input' trace_id=f9b58869-c78e-4b2b-916f-14ece4356ab4
...
thread 'test_case_3_8_crypto_hash_uuid' panicked at crates/ferroflux-testing/tests/case_3_8_crypto_hash_uuid_test.rs:71:10:
should have called /verify-crypto
```
