# FerroFlux Observability & Tracing Guide

This guide explains how to use the built-in observability system to monitor, debug, and analyze your workflows in FerroFlux.

## Overview

FerroFlux treats observability as a **first-class system**. Every execution flow is represented as a live `Trace` entity in the ECS, allowing for high-fidelity, real-time tracking and durable analytics.

## 1. Using the `trace` Tool

The `trace` tool is a primitive utility you can add to your YAML node definitions to capture state at specific points in a workflow.

### Syntax
```yaml
execution:
  - id: <step_id>
    tool: trace
    params:
      label: "<string>"         # A human-readable label for the trace point
      data: "{{ <expression> }}" # The data you want to capture (interpolated)
```

### Example
Capturing the response from an API call before it gets processed:
```yaml
execution:
  - id: fetch_user
    tool: http_client
    params:
      method: GET
      url: "https://api.example.com/users/1"
  
  - id: debug_user_data
    tool: trace
    params:
      label: "Raw API Response"
      data: "{{ steps.fetch_user.result }}"
```

## 2. Real-time Telemetry

FerroFlux emits real-time events via the `SystemEventBus`. These are primarily used for the Playground's live view but can be consumed by any internal system.

### Key Events
- **`NodeTelemetry`**: Emitted when a node finishes execution. Includes `execution_ms`, `success` status, and `trace_id`.
- **`EdgeTraversal`**: Emitted when data moves from one node to another.
- **`Log`**: Emitted by the `trace` tool or standard logger.

## 3. Durable Analytics

For "black-box" recording and long-term debugging, FerroFlux uses an `AnalyticsBatcher`.

- **Batching**: Events are collected and written to a database (DuckDB or ClickHouse) every 2 seconds or 1000 events.
- **Querying**: You can query the analytics database to see the full history of a `trace_id`.

## 4. Maintenance & Lifecycle

### Trace entities
- Every workflow start creates a `Trace` entity.
- This entity is updated as the workflow traverses the graph.
- **Cleanup**: The `Janitor` system automatically prunes `Trace` entities older than 1 hour to keep the engine fast.

### Redaction
Sensitive data should be handled carefully. The `trace` tool captures whatever balance of data you provide. In future versions, declarative redaction in YAML will allow automatic masking of secrets before they hit the analytics database.

## 5. Shadow Mode (Advanced)
*Note: This is a proposed feature based on the current infrastructure.*

Because of the ECS-native design, you can spawn a "Shadow Graph"—a separate set of entities that share the same logic but use "Tool Mocks" for safe simulation and observation without hitting real APIs.
