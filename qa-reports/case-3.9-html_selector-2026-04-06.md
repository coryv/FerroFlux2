# QA Report: Case 3.9 — HTML Selector Extraction

- **Status:** ✅ PASS
- **Workflow ID:** `html_selector`
- **Date:** 2026-04-06
- **Engine Version:** 0.1.0

## Summary
The workflow failed during the execution of the `core.utils.html` node. The engine was unable to map the tool's output to the defined node outputs due to an inconsistency in the platform definition file (`utils.html.yaml`).

## Evidence
- **Log Entry:** `ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: CEL execution error for explicit expression 'steps.extract.result': No such key: result trace_id=...`
- **Root Cause:** 
    - `platforms/core/utils.html.yaml` defines `returns: result: extracted`. This tells the engine to look for a field named `extracted` from the tool and map it to `steps.extract.result`.
    - `crates/ferroflux-tools/src/primitives/html.rs` returns `json!({ "result": results })`.
    - Because the tool returns `result` instead of `extracted`, the mapping fails, and the downstream `emit` step cannot find `steps.extract.result`.

## Reproduction
1.  Load `fixtures/html_selector.yaml`.
2.  Trigger `webhook_in` with payload `{"html": "<h1>Hello</h1>"}`.
3.  Observe CEL error in logs.

## Recommendation
Update `platforms/core/utils.html.yaml` to use the correct return mapping:
```yaml
execution:
  - id: extract
    ...
    returns:
## Resolution
The platform definition `platforms/core/utils.html.yaml` was updated to use `result: result` in the `returns` mapping, correctly matching the output field from the Rust tool implementation.

Verified with `cargo test --test case_3_9_html_selector_test`.
