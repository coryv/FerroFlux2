---
name: ferroflux-workflow-builder
description: Build a FerroFlux workflow from a description. Use this skill any time the user wants to create a workflow, build automation, wire nodes together, or produce a WAML file — including "build a workflow that...", "create a workflow for...", "make a WAML for...", "automate X using FerroFlux", or any request that describes a multi-step process to run on the FerroFlux platform. Do NOT use this for adding platform integrations or node definitions — use ferroflux-integration for that.
---

# FerroFlux Workflow Builder

Use this skill to turn a workflow description into a valid WAML (`WorkflowBlueprint`) YAML file.

## What You're Building

A workflow YAML file (`workflows/<name>.yaml`) with three sections:

- **`triggers`** — what starts the workflow (webhook, schedule, manual, SSE)
- **`nodes`** — the processing steps (actions, utils, third-party calls)
- **`edges`** — the wiring between nodes (data flow + control flow)

---

## Process

### Step 1 — Parse the Description

Identify:
1. **Trigger**: What starts this? (webhook call, schedule, manual run, SSE stream)
2. **Steps**: What happens in order? (HTTP calls, AI, branching, transforms, third-party actions)
3. **Data flow**: What data passes between steps and what are the key field names?
4. **Error paths**: Are there failure branches or fallback steps?

### Step 2 — Inventory Available Nodes

Core nodes are always available. For third-party platforms, verify the platform directory exists first:

```bash
ls platforms/
ls platforms/core/
```

**Core Triggers**

| Type | When to use |
|---|---|
| `core.trigger.webhook` | HTTP call arrives at a URL |
| `core.trigger.schedule` | Time-based (interval, daily, weekly) |
| `core.trigger.manual` | User-initiated run |
| `core.trigger.sse` | Server-Sent Events stream |
| `core.trigger.subflow` | Called as a subflow by another workflow |

**Core Actions**

| Type | When to use |
|---|---|
| `core.action.http` | Make any outgoing HTTP request |
| `core.action.agent` | Call an LLM (openai, anthropic, google, groq, mistral, ollama) |
| `core.action.switch` | Branch on a condition (Router) |
| `core.action.split` | Fan-out: iterate over each item in an array (Iterator) |
| `core.action.aggregate` | Fan-in: collect items into a batch (Accumulator) |
| `core.action.script` | Run a Rhai script for custom logic |
| `core.action.log` | Write a debug/info/warn message |
| `core.action.delay` | Sleep for N seconds |
| `core.action.set_var` | Store a value in workflow context |
| `core.action.get_var` | Read a value from workflow context |
| `core.action.subflow_output` | Return a value from a subflow |

**Core Utils** (data transformation, no flow control)

| Type | When to use |
|---|---|
| `core.utils.transform` | Apply a CEL template to reshape data |
| `core.utils.json` | Parse, stringify, or query JSON |
| `core.utils.text` | String manipulation (split, join, regex, etc.) |
| `core.utils.math` | Arithmetic and numeric operations |
| `core.utils.date` | Date/time formatting and arithmetic |
| `core.utils.html` | Parse or extract from HTML |
| `core.utils.xml` | Parse or extract from XML |
| `core.utils.graphql` | Execute a GraphQL query |
| `core.utils.crypto` | Hash, sign, or verify data |
| `core.utils.pdf_read` | Extract text from a PDF |
| `core.utils.pdf_write` | Generate a PDF |
| `core.logic.condition` | Evaluate a boolean condition |
| `core.manipulation.stats` | Compute stats over a dataset |

**Available Third-Party Platforms** (partial list — always `ls platforms/<id>/` to confirm nodes exist)

```
airtable, anthropic, asana, aws, azure-openai, bitbucket, clickup,
confluence, discord, dropbox, excel, freshdesk, gemini, github, gitlab,
gmail, google-drive, google_analytics, google_calendar, google_docs,
google_sheets, groq, hubspot, intercom, jira, linear, mailchimp,
microsoft_teams, mistral, monday, mongodb, mysql, notion, ollama_cloud,
onedrive, open-meteo, openai, outlook, paypal, pipedrive, postgresql,
posthog, redis, resend, s3, salesforce, segment, sendgrid, sentry,
shopify, slack, smtp, stripe, supabase, telegram, trello, twilio,
vercel, whatsapp, woocommerce, zendesk
```

To find the exact node type for a platform action:
```bash
ls platforms/<platform_id>/
# Files named action.<category>.<verb>.yaml give type: <platform>.<category>.<verb>
# Files named trigger.<category>.<event>.yaml give type: <platform>.<category>.<event>
```

### Step 3 — Assign Node IDs

Use short, descriptive snake_case IDs. The runtime converts these to stable UUIDs.

Examples: `webhook_in`, `parse_body`, `call_openai`, `check_status`, `send_slack`, `log_error`, `notify_email`

### Step 4 — Write the WAML

Save to `workflows/<descriptive-kebab-name>.yaml`. Create the directory if it doesn't exist.

**Full structure:**

```yaml
id: "optional-stable-id"          # omit if not needed
name: "Human-Readable Name"
description: "What this workflow does"

triggers:
  - id: <node_id>
    name: <display name>
    type: <platform.category.event>
    config:
      # Settings for this trigger node (see node's interface.settings)
      path: "/my-hook"             # for webhook
      method: POST

nodes:
  - id: <node_id>
    name: <display name>
    type: <platform.category.verb>
    config:
      # Settings for this node (see node's interface.settings)
      url: "https://api.example.com/endpoint"
      method: POST
    secret:                        # optional: inject an env var as a header
      lookup_key: MY_API_KEY       # env var name
      header_name: Authorization   # header to inject into
      template: "Bearer {}"        # format string ({} = the secret value)

edges:
  - source_id: <id>
    target_id: <id>
    source_handle: <port>          # optional — see handle conventions below
    target_handle: <port>          # optional
    label: <label>                 # optional — use for branch labels
```

### Step 5 — Wire Edges Correctly

**Handle conventions:**

| Scenario | source_handle | target_handle |
|---|---|---|
| Trigger body → action input | `body` | *(omit)* |
| Trigger query → action | `query` | *(omit)* |
| Trigger headers → action | `headers` | *(omit)* |
| Action success flow | `Success` | *(omit)* |
| Action error flow | `Error` | *(omit)* |
| Named data output → named input | output port name | input port name |
| Router branch | branch label (e.g., `"true"`, `"false"`, `"default"`) | *(omit)* |

**Rules:**
- Triggers have no inputs — never create edges pointing *to* a trigger
- Every action needs flow (an edge arriving at it) or it will never execute
- Data edges carry payload; flow edges carry execution signal
- When a trigger emits `body` and the next node needs data, wire `body → body` or rely on context (context accumulates automatically)

### Step 6 — Validate

```bash
# Structural parse check
cargo run --bin ferroflux-validate -- workflows/<name>.yaml
```

If validation isn't available for workflow files, manually verify:
- [ ] All `source_id` and `target_id` values reference real node IDs
- [ ] `type` fields match actual node definition files in `platforms/`
- [ ] Trigger has no incoming edges
- [ ] Every non-trigger node has at least one incoming edge
- [ ] No circular dependencies (unless intentionally using subflows)

---

## Config vs Edges — Key Distinction

- **Config**: Static values the user sets once (URLs, model names, templates, paths)
- **Edges**: Dynamic data wired from a previous node's output at runtime

Config in a workflow node is equivalent to `settings.*` in a node definition. Do **not** use CEL `inputs.*` in workflow config — inputs come from edges, not config.

```yaml
# CORRECT — static config, data wired via edge
nodes:
  - id: call_api
    type: core.action.http
    config:
      url: "https://api.example.com/process"
      method: POST

edges:
  - source_id: webhook_in
    source_handle: body
    target_id: call_api

# WRONG — do not interpolate inputs in workflow config
    config:
      body: inputs.body   # inputs.* is for node definitions, not workflow configs
```

---

## Common Workflow Patterns

### Webhook → Action → Notify

```yaml
triggers:
  - id: webhook_in
    name: Incoming Webhook
    type: core.trigger.webhook
    config:
      path: /my-hook
      method: POST

nodes:
  - id: process
    name: Process Data
    type: core.action.http
    config:
      url: "https://api.example.com/process"
      method: POST

  - id: notify
    name: Send Notification
    type: slack.messages.send_dm
    config:
      user_id: "U12345"
      text: "Done!"

edges:
  - source_id: webhook_in
    source_handle: body
    target_id: process
  - source_id: process
    source_handle: Success
    target_id: notify
```

### Schedule → Fetch → AI → Store

```yaml
triggers:
  - id: timer
    name: Daily Trigger
    type: core.trigger.schedule
    config:
      mode: daily
      time: "09:00"

nodes:
  - id: fetch_data
    type: core.action.http
    config:
      url: "https://api.example.com/data"
      method: GET

  - id: analyze
    type: core.action.agent
    config:
      provider: anthropic
      model: claude-sonnet-4-6
      system_instruction: "Analyze the following data and summarize key insights."

  - id: store_result
    type: postgresql.records.insert
    config:
      table: analyses

edges:
  - source_id: timer
    source_handle: Success
    target_id: fetch_data
  - source_id: fetch_data
    source_handle: Success
    target_id: analyze
  - source_id: analyze
    source_handle: Success
    target_id: store_result
```

### Branch on Condition

```yaml
nodes:
  - id: check
    type: core.action.switch
    config:
      rules:
        - condition: "val.status == 'urgent'"
          output: urgent
        - condition: default
          output: normal

  - id: urgent_path
    type: slack.messages.send_dm
    config:
      text: "URGENT: Action required!"

  - id: normal_path
    type: core.action.log
    config:
      level: INFO
      message: "Routine item processed."

edges:
  - source_id: check
    source_handle: urgent
    target_id: urgent_path
  - source_id: check
    source_handle: normal
    target_id: normal_path
```

### Fan-out with Split + Aggregate

```yaml
nodes:
  - id: split_items
    type: core.action.split
    config: {}

  - id: process_item
    type: core.action.http
    config:
      url: "https://api.example.com/item"
      method: POST

  - id: collect_results
    type: core.action.aggregate
    config:
      batch_size: 0    # 0 = wait for all

edges:
  - source_id: split_items
    source_handle: item
    target_id: process_item
  - source_id: process_item
    source_handle: Success
    target_id: collect_results
```

---

## Checklist Before Finishing

- [ ] Every node has a unique `id`
- [ ] Every `type` references a file that exists under `platforms/`
- [ ] Trigger has no incoming edges
- [ ] Every non-trigger node has at least one incoming edge
- [ ] `source_handle` set when routing specific data ports (body, query, Success, Error)
- [ ] Secrets declared in `secret:` block for any nodes using API keys
- [ ] Workflow saved to `workflows/<name>.yaml`
- [ ] `cargo run --bin ferroflux-validate -- workflows/<name>.yaml` passes
