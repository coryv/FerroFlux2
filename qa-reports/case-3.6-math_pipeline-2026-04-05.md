# FerroFlux QA Report

**Date:** 2026-04-05
**Workflow:** `webhook -> math (mul) -> math (add) -> verify_result (HTTP)`
**Nodes tested:** `core.trigger.webhook`, `core.utils.math`, `core.action.http`
**Test file:** `crates/ferroflux-testing/tests/case_3_6_math_pipeline_test.rs`

---

## Result: ✅ PASS

---

## Workflow Under Test

| Step | Node Type | HTTP Call(s) |
|------|-----------|--------------|
| Trigger | `core.trigger.webhook` | — |
| 1 | `core.utils.math` (mul) | — |
| 2 | `core.utils.math` (add) | — |
| 3 | `core.action.http` | POST /verify-total |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| `math_multiply` executes | ✅ | Synchronization Gate: All inputs present |
| `math_multiply` emits to `result` | ✅ | Value `108.0` correctly propagated |
| `math_add` executes | ✅ | Synchronization Gate cleared with all inputs |
| `verify-total` called | ✅ | Received body: `108.0` |

## Resolution Details

The initial failure was caused by a combination of platform-level mapping errors and engine-level synchronization bugs:

### 1. Platform Mapping Fixes
- **Variable Alignment**: Corrected `core.utils.math` to reference the mapped `math_res` variable instead of the non-existent `steps.calc.result`.
- **Dynamic Inputs**: Updated `core.action.http` to declare `body` and `headers` as valid inputs, enabling data passage via WAML templates.
- **CEL Port Quoting**: Wrapped port names (e.g., `'result'`, `'Exec'`) in quotes to prevent CEL from attempting to resolve them as context variables.

### 2. Engine Synchronization Fixes
- **InputBuffer Loss**: Resolved a race condition where the `InputBuffer` component would be initialized multiple times during a single tick's ingestion, leading to data loss for parallel ports.
- **WorkDone Signal**: Fixed a bug where the synchronization system failed to signal progress (`WorkDone=true`) during node execution, causing the test harness to stall prematurely.

### 3. Security Bypass (Internal IPs)
- **SSRF Verification**: Enabled `FERROFLUX_ALLOW_INTERNAL_IPS=1` during test execution to allow calls to the local `wiremock` server on `127.0.0.1`.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | PASS |
| Cargo test duration | 0.78s |
| HTTP calls expected | 1 |
| HTTP calls received | 1 |
| Assertions | 4/4 passed |

## Raw Test Output (Success)

```
Tick 0: WorkDone=true
Tick 1: WorkDone=true
...
2026-04-05T14:42:18.984784Z  INFO ferroflux_core::systems::pipeline: Synchronization Gate: All inputs present, executing entity=5v1
>>> Success! Received verify-total call at tick 6
>>> Received Body: 108.0
>>> Case 3.6: Math Operations Pipeline - PASSED
test test_case_3_6_math_pipeline ... ok
```
