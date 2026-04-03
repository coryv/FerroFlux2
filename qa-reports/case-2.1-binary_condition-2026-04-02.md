# FerroFlux QA Report

**Date:** 2026-04-02
**Workflow:** Binary Condition (If/Else)
**Nodes tested:** `core.trigger.webhook`, `core.action.script`, `core.logic.condition`, `core.action.log`
**Test file:** `crates/ferroflux-testing/tests/binary_condition_test.rs`

---

## Result: ❌ FAIL (Synchronization Issue)

---

## Workflow Under Test

| Step | Node Type | Input Port(s) | Expected Logic |
|------|-----------|---------------|----------------|
| Trigger | `core.trigger.webhook` | — | Fires immediately |
| 1 | `core.action.script` | Exec | Returns 150 |
| 2 | `core.action.script` | Exec | Returns 100 |
| 3 | `core.logic.condition` | a, b, Exec | Compares 150 > 100 |

## Assertions

| Assertion | Status | Evidence |
|-----------|--------|----------|
| Script nodes execute | ✅ | `Moved ticket source=0v1 target=1v1` |
| Data arrives for a and b | ✅ | Debug logs show tickets moved to `a` and `b` handles |
| Condition evaluates True | ❌ | `CEL execution failed during resolution ... no such key: a` |

## Engine Contract Violations

> [!CAUTION]
> **Input Synchronization Race**: In the FerroFlux engine, a node begins execution the moment its `Exec` port receives a ticket. In a parallel-trigger workflow like 2.1, the `Exec` signal reaches the `Binary Condition` node *before* the data signals from the script nodes achieve their travel.
> 
> As a result, the node evaluates with `a=null` and `b=null`, leading to a resolution failure and incorrect routing. The engine currently lacks a synchronization gate to ensure all expected data ports are populated before `execution` block resolution occurs.

## Metrics

| Metric | Value |
|--------|-------|
| Test result | FAIL |
| Cargo test duration | ~1.1s |
| Assertions | 2/3 passed |

## Raw Test Output

```
2026-04-02T02:46:44.374908Z  INFO ferroflux_core::systems::pipeline: Processing inbox item entity=2v1 port=Some("a")
2026-04-02T02:46:44.375405Z DEBUG ferroflux_core::systems::pipeline::resolution: CEL execution failed during resolution, using literal string expression="inputs.a" error=No such key: a
```
