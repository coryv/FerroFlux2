# QA Report: Case 3.12 — GraphQL Query

- **Status:** ✅ PASS
- **Workflow ID:** `graphql_query`
- **Date:** 2026-04-06

## Summary
The GraphQL client failed to execute its status checking logic due to an invalid CEL expression mapping in the platform definition file (`utils.graphql.yaml`).

## Evidence
- **Log Entry:** `ERROR ferroflux_core::systems::pipeline: Pipeline execution failed: CEL execution error for explicit expression 'val < 400': Undeclared reference to 'val' trace_id=...`
- **Root Cause:** 
    - `platforms/core/utils.graphql.yaml` uses `=val < 400` inside a `logic` tool rule.
    - The `logic` tool's configuration in the YAML is meant to be a declarative JSON object, but the node is attempting to use CEL for a variable that only exists inside the tool's execution scope (`val`), not the workflow's scope.
    - The `core.trigger.manual` trigger also requires an `event` context (e.g., `user_id`) which must be explicitly provided in an automated test environment.

## Reproduction
1.  Load `fixtures/graphql_query.yaml`.
2.  Trigger manually using `TestHarness`.
3.  Observe CEL error in logs when the `check_status` node attempts to run.

## Recommendation
Update `platforms/core/utils.graphql.yaml` to use a structured condition for its logic step:
```yaml
      rules:
        - condition:
            operator: "<"
            value: 400
          output: Success
```
## Resolution
The platform definition `platforms/core/utils.graphql.yaml` was updated to use structured conditions in the `check_status` logic step. This avoids invalid CEL evaluations of variables that are internal to the `logic` tool.

```yaml
      rules:
        - condition:
            operator: "<"
            value: 400
          output: Success
        - condition:
            operator: ">="
            value: 400
          output: Error
```

Verified with `cargo test --test case_3_12_graphql_query_test -- --nocapture`.
