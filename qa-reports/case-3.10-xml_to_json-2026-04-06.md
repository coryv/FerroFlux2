# QA Report: Case 3.10 — XML to JSON Pipeline

- **Status:** ✅ PASS
- **Workflow ID:** `xml_to_json`
- **Date:** 2026-04-06

## Summary
The `core.utils.xml` utility failed to map its internal tool result to the node's output port. In XML transformation scenarios, the engine currently encounters configuration mismatches in the platform metadata.

## Evidence
- **Log Entry:** `ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: CEL execution error for explicit expression 'steps.parse.result': No such key: result trace_id=...`
- **Root Cause:** 
    - `platforms/core/utils.xml.yaml` defines `returns: result: xml_result`.
    - `crates/ferroflux-tools/src/primitives/xml.rs` returns `json!({ "result": json_val })`.
    - Mapping `result: xml_result` fails because the tool output key is `result`.

## Reproduction
1.  Load `fixtures/xml_to_json.yaml`.
2.  Trigger `webhook_in` with XML payload.
3.  Observe CEL error in logs.

## Recommendation
Update `platforms/core/utils.xml.yaml` to:
```yaml
execution:
  - id: parse
    ...
    returns:
## Resolution
The platform definition `platforms/core/utils.xml.yaml` was updated to use `result: result` in the `returns` mapping, correctly matching the output field from the Rust tool implementation.

Verified with `cargo test --test case_3_10_xml_to_json_test`.
