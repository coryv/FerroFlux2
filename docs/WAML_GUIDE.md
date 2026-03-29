# FerroFlux WAML API Guide

Workflow Architecture Markup Language (WAML) is the declarative language used to define automation flows within the FerroFlux engine. This document serves as the canonical API reference for generating valid WAML payloads and testing them against the headless engine.

> [!NOTE]
> WAML is strictly a backend logical definition. It does not contain layout, coordinate, or UI-rendering data. All visual aspects of workflows are determined by auto-layout heuristics in the client application.

---

## 1. Document Structure

A valid WAML document is a YAML configuration file containing metadata, triggers, nodes, and edges.

```yaml
name: "My Workflow"
description: "Handles user onboarding and Slack notification."
triggers: []
nodes: []
edges: []
```

---

## 2. Defining Nodes

Nodes are the operational units of the workflow. Every node must have a `type` that maps exactly to a definition in the `platforms/` directory (e.g., `slack.messages.send_dm`).

### Node Schema
- **`id`**: A unique string identifier for the node within the document. (e.g., `"slack_out"`).
- **`name`**: A human-readable display name.
- **`type`**: The fully qualified ID of the platform integration node. (e.g., `"core.action.log"`).
- **`config`**: Key-value pairs matching the `settings` schema in the corresponding platform YAML definition.

### Example Node
If `platforms/core/action.log.yaml` expects a `level` and a `message`, the node definition looks like this:
```yaml
nodes:
  - id: "slack_out"
    name: "Post to Slack"
    type: "slack.messages.send_dm"
    config:
      user_id: "U12345"
      text: "New onboarding event recorded."
```

---

## 3. Defining Triggers

Triggers instantiate the workflow. They follow the same general schema as `nodes` but are listed under the `triggers` block.

### Example Trigger
```yaml
triggers:
  - id: "webhook_in"
    name: "Incoming Webhook"
    type: "core.trigger.webhook"
    config:
        path: "/test-webhook"
```

---

## 4. Port-Based Edge Mapping

FerroFlux uses a strict "Inbox/Outbox" architecture for data movement. Edges do not simply connect node A to node B; they explicitly map a **source port** (handle) to a **target port** (handle).

### Edge Schema
- **`source_id`**: The `id` of the upstream node/trigger.
- **`source_handle`**: The specific named port emitting data (defined in the platform YAML, typically `"body"` or `"Success"`).
- **`target_id`**: The `id` of the downstream node.
- **`target_handle`**: The explicit inbox port on the downstream node (typically `"body"` or `"Exec"`).

### Example Edge
```yaml
edges:
  - source_id: "webhook_in"
    source_handle: "body"
    target_id: "slack_out"
    target_handle: "body"
```

---

## 5. Handlebars Data Resolution

Nodes can dynamically access data from upstream components using Handlebars syntax (`{{ ... }}`). When an upstream node emits data via a port, the engine merges that data into the local execution context via the "Flow Bus" architecture.

### Context Access Patterns
1.  **Global Port Data (`get`)**: You can use the `get` helper to retrieve data that was merged from an upstream edge map.
    *   *Usage*: `{{ get 'body.text' }}` (Retrieves the `text` field from the data envelope that arrived on the `body` target handle).
2.  **Platform Execution State (`steps`)**: Multi-step platform actions retain their internal operational state across steps.
    *   *Usage*: `{{ steps.open_dm.body.channel.id }}` (Access local step variables before they are emitted to the rest of the flow).
3.  **Local Configuration (`inputs`)**: Dynamic resolution of the node's own config object.
    *   *Usage*: `{{ inputs.user_id }}` (Retrieves the evaluated config field).
4.  **Raw Trigger Payload (`event`)**: For `core.trigger.webhook`, the raw payload is often nested inside an `event` wrapper during execution. Though normally data emitted on the `body` is accessed via `get`, raw contexts can occasionally be referenced depending on how the node defines its execution map.

### Example Resolution
```yaml
nodes:
  - id: "log_out"
    name: "Log Message"
    type: "core.action.log"
    config:
      level: "INFO"
      message: "Received user profile event for: {{ get 'body.user_metadata.name' }}"
```

---

## 6. Integration Testing Principles

When writing automated QA integration tests in Rust (`crates/ferroflux-testing/tests/`):

1.  **Always Map Ports**: Failure to provide explicit `source_handle` and `target_handle` will result in the edge failing to initialize in the `GraphTopology`, and downstream nodes will never execute.
2.  **Mock Exact JSON Structures**: The `TestHarness` utilizes `wiremock`. Ensure your mock stubs reflect the *exact* JSON output expected by the node's specific platform YAML. `HttpClientTool` extracts `.body`, so the mock must yield valid matching JSON to avoid template resolution panics in the execution phase.
3.  **Do Not Touch the Engine**: If a workflow fixture behaves incorrectly and the issue traces to a data resolution error in `ferroflux_core`, **log the issue**. Do not attempt to refactor the pipeline execution logic. It is the job of the core engineers to ensure the engine meets the API contract defined by this document.
