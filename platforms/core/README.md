# Core Integration Guide

The Core platform provides the fundamental building blocks for FerroFlux workflows. Unlike API-based platforms, Core tools interact directly with the engine's runtime to handle flow control, state, and complex data transformations.

## Setup & Authentication
The Core platform is built-in and does not require external authentication.

---

## Flow Control Tools

### `delay`
Pauses the workflow for a specified duration.
- **Inputs**: `Exec` (flow).
- **Settings**: `duration` (string, e.g., "5s", "10m", "1h").

**Example:**
```waml
- step: wait_for_process
  call: core.delay
  with:
    duration: "30s"
```

### `switch`
Routes the workflow to different output ports based on conditional logic.
- **Inputs**: `data` (any).
- **Settings**: `rules` (list of conditions and target ports).

**Example:**
```waml
- step: route_by_status
  call: core.switch
  with:
    data: steps.get_status.result
    rules:
      - condition: "val == 'active'"
        output: "ProcessActive"
      - condition: "val == 'pending'"
        output: "HandlePending"
      - condition: default
        output: "Error"
```

### `split`
Iterates over an array, emitting each item individually to the `item` port.
- **Inputs**: `array` (array).
- **Outputs**: `item` (flow/any), `Done` (flow).

**Example:**
```waml
- step: iterate_users
  call: core.split
  with:
    array: steps.get_users.list
```

### `aggregate`
Collects individual items into a batch (array). Useful for batching records before a database insert.
- **Inputs**: `item` (any).
- **Settings**: `batch_size` (number).
- **Outputs**: `batch` (array), `Exec` (flow - fired when batch is ready).

**Example:**
```waml
- step: batch_records
  call: core.aggregate
  with:
    item: steps.processed_item.data
    batch_size: 50
```

---

## State & Variable Tools

### `set_var` / `get_var`
Stores and retrieves values from the workflow's global memory.
- **Settings**: `variable_name` (string).

**Example:**
```waml
- step: save_state
  call: core.set_var
  with:
    variable_name: "last_processed_id"
    value: steps.current_item.id

- step: load_state
  call: core.get_var
  with:
    variable_name: "last_processed_id"
```

---

## Advanced Logic Tools

### `http`
Makes generic HTTP requests. Use this for APIs that don't have a native integration yet.
- **Settings**: `url`, `method`, `body`, `connection`.

**Example:**
```waml
- step: custom_api_call
  call: core.http
  with:
    url: "https://api.example.com/v1/resource"
    method: "POST"
    body: 
      name: inputs.user_name
    connection: "optional_stored_connection_id"
```

### `script`
Executes custom Rhai (Rust-like) scripts for complex data manipulation.
- **Settings**: `script` (textarea).

**Example:**
```waml
- step: complex_transform
  call: core.script
  with:
    script: |
      let data = get("input_data");
      if data.score > 80 {
        return "A";
      } else {
        return "B";
      }
```

### `agent`
Invokes an AI Agent (OpenAI, Anthropic, or Gemini) to process a prompt.
- **Settings**: `provider`, `model`, `prompt`, `system_instruction`.

**Example:**
```waml
- step: summarize_text
  call: core.agent
  with:
    provider: "openai"
    model: "gpt-4o"
    prompt: "Summarize the following text: " + steps.get_text.content
    system_instruction: "You are a professional editor."
```

---

## Utility Tools

| Tool | Example WAML |
| --- | --- |
| `utils.json` | `call: core.utils.json` with `operation: "query"`, `path: "$.user.id"` |
| `utils.text` | `call: core.utils.text` with `operation: "regex_replace"`, `pattern: "\\s+"`, `replacement: "_"` |
| `utils.date` | `call: core.utils.date` with `operation: "format"`, `value: "now"`, `format: "YYYY-MM-DD"` |
| `utils.math` | `call: core.utils.math` with `operation: "sum"`, `values: [1, 2, 3]` |
| `utils.crypto` | `call: core.utils.crypto` with `operation: "hash"`, `algorithm: "sha256"`, `value: "secret"` |
| `utils.html` | `call: core.utils.html` with `operation: "scrape"`, `selector: "table.data tr"` |
| `utils.pdf_read` | `call: core.utils.pdf_read` with `file: steps.get_file.content` |

---

## Triggers

### `manual`
Fires when the user clicks 'Run' in the FerroFlux dashboard.

### `scheduler`
Fires based on time intervals.
```waml
trigger:
  call: core.trigger.scheduler
  with:
    cron: "0 9 * * 1-5" # 9 AM Monday-Friday
```

### `webhook`
Triggers the workflow via an external HTTP call.
```waml
trigger:
  call: core.trigger.webhook
  with:
    path: "/github-webhook"
    method: "POST"
```
