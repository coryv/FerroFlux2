# FerroFlux Node Creation Guide

## Introduction to Data-Driven Nodes

FerroFlux utilizes a **Data-Driven Architecture** where nodes are defined declaratively using YAML. This approach decouples the definition of logic (WHAT a node does) from the implementation (HOW code executes it). 

This guide explains how to author `.yaml` node definitions to create powerful, reusable components without compiling Rust code.

---

## Node Anatomy

A Node Definition consists of four main sections:
1. **Meta**: Metadata describing the node.
2. **Interface**: Inputs, Outputs, and Settings.
3. **Execution**: A linear sequence of distinct steps using atomic Tools.
4. **Routing**: (Optional) Logic for branching control flow based on step results.

### 1. Metadata (`meta`)

```yaml
meta:
  id: my.package.node_name  # Unique identifier (namespaced)
  name: "My Custom Node"    # Human-readable name
  category: "Utilities"     # Palette category
  type: Action              # Node Type: Action, Trigger, Utility
  description: "Does something amazing"
  version: "1.0.0"
```

### 2. Interface (`interface`)
Defines how the node interacts with the outside world.

```yaml
interface:
  inputs:
    - name: data_in
      type: string
  
  outputs:
    - name: result
      type: string
    - name: Success
      type: flow

  settings:
    - name: mode
      label: "Recurrence"
      type: select
      default: "once"
      options:
        - label: "Once"
          value: "once"
        - label: "Interval"
          value: "interval"
    
    - name: interval
      label: "Every (ms)"
      type: number
      default: 1000
      min: 100
      show_if: "mode == 'interval'"
```

#### Advanced Settings Features
- **`show_if`**: Controls visibility in the Property Inspector. Supports equality (`==`), inequality (`!=`), and logical OR (`||`).
  - Example: `show_if: "mode == 'daily' || mode == 'weekly'"`
- **Numeric Constraints**: `min`, `max`, and `step` can be applied to `number` types.
- **Dynamic Options**: (Coming soon) `options_provider` can link to a backend function.

### 3. Execution Pipeline (`execution`)
The core logic. It is a list of steps. Each step executes a primitive **Tool** (see `tools_reference.md`).

```yaml
execution:
  - id: fetch_data                  # Step ID (referenced later as steps.fetch_data)
    tool: http_client               # Tool to execute
    params:
      method: GET
      url: "https://api.example.com/data"
    returns:
      status: status_code           # Map tool output "status" -> context "status_code"
```

### 4. Routing Logic (`routing`)
Conditional flow control.

```yaml
routing:
  match: "{{ steps.fetch_data.status }}" # Template expression
  cases:
    "200":                            # If match == "200"
      - tool: emit
        params:
          port: result
          value: "{{ steps.fetch_data.body }}"
    default:
      - tool: emit
        params:
          port: Success
```

---

## Context & Templating

Variables in your YAML definition are resolved using **Handlebars-style** syntax `{{ variable }}`.

### Available Context Namespaces:
- **`settings`**: Configuration values set by the user in the Property Inspector.
- **`platform`**: Shared configuration from the associated Platform (e.g., `{{ platform.base_url }}`).
- **`steps`**: Results from previous execution steps. (e.g., `{{ steps.step_id.output_key }}`)
- **Root Context**: Values passed into the node from previous nodes are available at the root (e.g., `{{ my_variable }}`).

### Type Preservation
FerroFlux is "type-aware" during interpolation. 
- If a parameter value is a **single** template expression like `json: "{{ steps.api.body }}"`, FerroFlux will preserve the underlying type (Object, Array, Number).
- Mixed content like `msg: "Status is {{ steps.api.status }}"` will always result in a **string**.

---

## Best Practices

1. **Namespace IDs**: Use `platform.feature.action` format for IDs to avoid collisions (e.g., `openai.chat.completions`).
2. **Atomic Steps**: Break complex logic into multiple small steps using generic tools like `json_query` or `math`.
3. **Handle Errors**: Always provide a `Success` or `Error` port to ensure the node doesn't fail silently.
4. **Use Platforms**: Don't hardcode API keys or base URLs. Use `{{ platform.config_key }}`.
