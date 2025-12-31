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
    - name: error
      type: flow

  settings:
    - name: retry_count
      label: "Max Retries"
      type: number
      default: 3
```

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
  match: "{{ steps.fetch_data.status }}" # Handlebars template expression
  cases:
    200:                            # If match == "200" (or 200)
      - tool: emit
        params:
          port: result
          value: "{{ steps.fetch_data.body }}"
    default:
      - tool: emit
        params:
          port: error
```

---

## Context & Templating

Variables in your YAML definition are resolved using **Handlebars** syntax `{{ variable }}`.

### Available Context Namespaces:
- **`inputs`**: Values passed into the node's input ports. (e.g., `{{ inputs.data_in }}`)
- **`settings`**: Configuration values set by the user. (e.g., `{{ settings.retry_count }}`)
- **`steps`**: Results from previous execution steps. (e.g., `{{ steps.step_id.output_key }}`)
- **`platform`**: (If applicable) Configuration from the associated Platform (e.g., `{{ platform.api_key }}`).

### Type Preservation
If a parameter value is a **single** template expression like `json: "{{ inputs.my_object }}"`, FerroFlux will attempt to preserve the underlying type (Object, Array, Number) instead of casting it to a string. 
Mixed content like `msg: "Value is {{ inputs.val }}"` will always result in a string.

---

## Best Practices

1. **Namespace IDs**: Use `platform.feature.action` format for IDs to avoid collisions (e.g., `openai.chat.completions`).
2. **Atomic Steps**: Break complex logic into multiple small steps.
3. **Handle Errors**: Always provide a `default` or error case in routing to ensure the node doesn't fail silently.
4. **Use Platforms**: Don't hardcode API keys. Use `{{ platform.auth }}` and define a Platform Definition.
