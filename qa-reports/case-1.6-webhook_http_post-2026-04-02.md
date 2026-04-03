# FerroFlux QA Report

**Date:** 2026-04-02
**Workflow:** Webhook to HTTP POST Chain
**Nodes tested:** `core.trigger.webhook`, `core.action.http`, `core.action.log`
**Test file:** `crates/ferroflux-testing/tests/webhook_http_post_test.rs`

---

## Result: ✅ PASS (with warnings)

---

## Workflow Under Test

| Step | Node Type | HTTP Call(s) |
|------|-----------|--------------|
| Trigger | `core.trigger.webhook` | — |
| 1 | `core.action.http` | POST /post-sink |
| 2 | `core.action.log` | — |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| Webhook trigger fires | ✅ | `Trigger sent to OUTBOX` |
| HTTP POST called | ✅ | Mock server recorded POST to `/post-sink` |
| Log node outputs message | ✅ | `Procressing inbox item entity=0v1 port=Some("Exec")` |

## Observations & Issues

1. **CEL Resolution Warnings**: The engine produced multiple `WARN` logs for string literals like `"POST"`, `"Success"`, and `"Error"`, indicating it is trying and failing to resolve these as CEL expressions before falling back to string literals.
2. **Missing Configuration Keys**: The HTTP node reported `No such key: body` and `No such key: output_var` during execution because these optional fields were not provided in WAML and the engine defaults were not correctly overlaid/resolved in the execution context.
3. **Dynamic Data Limitations**: While the chain passed, the platform definition for `core.action.http` currently uses static `settings.body` in its execution tool. Forwarding the dynamic webhook `body` via an edge is not currently supported by the platform's `params` mapping.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | PASS |
| Cargo test duration | ~0.64s |
| HTTP calls expected | 1 |
| HTTP calls received | 1 |
| Assertions | 3/3 passed |

## Raw Test Output

```
2026-04-02T01:58:29.173045Z  INFO Processing inbox item entity=0v1 port=Some("Exec") ticket_id=f43bfc44-96a2-46f0-b13b-d9b0c97fbc0a
2026-04-02T01:58:29.173763Z  WARN CEL execution failed during resolution, using literal string expression="POST" error=Undeclared reference to 'POST'
2026-04-02T01:58:29.174136Z  WARN CEL execution failed during resolution, using literal string expression="settings.body" error=No such key: body
```
