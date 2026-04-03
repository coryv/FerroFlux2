# FerroFlux QA Report

**Date:** 2026-04-02
**Workflow:** Split Array (Fan-Out Only)
**Nodes tested:** `core.action.script`, `core.action.split`, `core.action.log`
**Test file:** `crates/ferroflux-testing/tests/split_array_test.rs`

---

## Result: ❌ FAIL (Engine Contract Violation)

---

## Workflow Under Test

| Step | Node Type | HTTP Call(s) |
|------|-----------|--------------|
| Trigger | `core.trigger.webhook` | — |
| 1 | `core.action.script` | — |
| 2 | `core.action.split` | N/A (Internal Tool) |
| 3 | `core.action.log` | — |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| Script node returns array | ✅ | `Moved ticket source=0v1 target=1v1` |
| Split tool executed | ❌ | `ERROR: Pipeline execution failed: Tool not found: split` |
| Log node processes 3 items | ❌ | Never reached |

## Engine Contract Violations

> [!IMPORTANT]
> **Missing Tool Implementation**: The platform definition `core.action.split` specifies the use of a tool named `split`. However, this tool is not registered in the FerroFlux engine's `ToolRegistry`. 
> 
> The engine fails with `Tool not found: split` when attempting to execute the iteration logic, effectively blocking any workflow that requires array fan-out.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | FAIL |
| Cargo test duration | ~1.1s |
| Assertions | 1/3 passed |

## Raw Test Output

```
2026-04-02T02:45:13.186031Z  INFO ferroflux_core::systems::pipeline: Processing inbox item entity=0v1 port=Some("Exec")
2026-04-02T02:45:13.195865Z DEBUG transport_worker: ferroflux_core::systems::transport: Moved ticket source=0v1 target=1v1 port=Some("result") target_handle=Some("array")
2026-04-02T02:45:13.195923Z  INFO ferroflux_core::systems::pipeline: Processing inbox item entity=1v1 port=Some("array")
2026-04-02T02:45:13.196662Z ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: Tool not found: split trace_id=...
```
