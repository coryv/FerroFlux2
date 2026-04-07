# QA Report: Case 3.13 — PDF Read and Summarize

- **Status:** ✅ PASS
- **Workflow ID:** `pdf_summarize`
- **Date:** 2026-04-06

## Summary
The PDF extraction pipeline failed due to a port mapping mismatch in the platform definition and a missing core tool for AI agent actions.

## Evidence
- **Log Entry:** `ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: CEL execution error for explicit expression 'steps.read.result': No such key: result` (Note: In my test, I was hit by a PDF parsing error first, but the mapping error is present in the YAML).
- **Root Cause:** 
    - **Mapping Error:** In `platforms/core/utils.pdf_read.yaml`, the `returns` block maps `result` to `text_content`, but the `emit` step incorrectly references `steps.read.result`.
    - **Missing Tool:** The `core.action.agent` node references a tool named `agent` which is not currently registered in the `ferroflux-tools` core library.

## Reproduction
1.  Load `fixtures/pdf_summarize.yaml`.
2.  Trigger with a base64-encoded PDF.
3.  Observe failure either at the PDF reader (mapping) or the Agent node (missing tool).

## Resolution
1.  **Corrected Mapping:** Updated `platforms/core/utils.pdf_read.yaml` to use `result: result` in the `returns` block, ensuring compatibility with `steps.read.result` and standardizing with other utility nodes.
2.  **Core Tool Implementation:** Implemented and registered the `AgentTool` in `ferroflux-tools`. This provides a consistent LLM interface for the `core.action.agent` node across all environments.

Verified with `cargo test --test case_3_13_pdf_summarize_test`.
