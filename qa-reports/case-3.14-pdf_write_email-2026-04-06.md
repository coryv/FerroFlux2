# QA Report: Case 3.14 — PDF Write and Email

- **Status:** ✅ PASS
- **Workflow ID:** `pdf_email`
- **Date:** 2026-04-06

## Summary
The PDF generation and emailing pipeline failed due to a port mapping mismatch in the platform definition file.

## Evidence
- **Log Entry:** `ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: CEL execution error for explicit expression 'steps.write.result': No such key: result`
- **Root Cause:** In `platforms/core/utils.pdf_write.yaml`, the `returns` block maps `result` to `pdf_base64`, but the `emit` step incorrectly references `steps.write.result`. This key does not exist because the tool's `result` was mapped to `pdf_base64` in the scope of the node.
- **Verification:** Both the `pdf_reader` and `pdf_write` nodes show this same pattern error.

## Reproduction
1.  Load `fixtures/pdf_email.yaml`.
2.  Trigger via webhook with content string.
3.  Observe failure in the `pdf_gen` node due to `No such key: result`.

## Resolution
- **Corrected Mapping:** Updated `platforms/core/utils.pdf_write.yaml` to use `result: result` in the `returns` block, ensuring compatibility with `steps.write.result` and standardizing with other utility nodes.

Verified with `cargo test --test case_3_14_pdf_email_test`.
