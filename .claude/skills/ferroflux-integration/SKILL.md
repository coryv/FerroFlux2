---
name: ferroflux-integration
description: Build a FerroFlux platform integration from an API. Use this skill any time the user wants to add a new integration, connector, or platform to FerroFlux — including when they say "add integration", "create a platform", "build a connector", "integrate with X", "add X support", or provide an API docs URL and ask you to build nodes for it. Always use this skill when creating or modifying YAML files under the platforms/ directory.
---

# FerroFlux Integration Builder

Use this skill to create a complete FerroFlux integration for a third-party API.

## What You're Building

Every integration consists of two types of files saved under `platforms/<platform_id>/`:

1. **Platform file** (`<platform_id>.yaml`) — shared config (base URL, auth headers)
2. **Node files** (`action.<category>.<verb>.yaml` or `trigger.<category>.<event>.yaml`) — one per API operation

---

## Scripts

Four helper scripts live in `.claude/skills/ferroflux-integration/scripts/`. **Always run these from the FerroFlux2 project root.** They eliminate boilerplate, catch bugs before the slow Rust compiler runs, and prevent duplication when expanding existing platforms.

| Script                 | When to use                                                                                         |
| ---------------------- | --------------------------------------------------------------------------------------------------- |
| `scaffold-platform.sh` | Creating a new platform — generates a structurally-valid `<id>.yaml` skeleton                       |
| `scaffold-node.sh`     | Creating each node — generates a structurally-valid action or trigger YAML skeleton                 |
| `pre-lint.sh`          | After editing any YAML — catches `{{ settings.x }}`/`{{ inputs.x }}` bugs the Rust validator misses |
| `inventory.sh`         | Before expanding an existing platform — shows what nodes already exist                              |

```bash
# Quick reference
bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh <id> "<Name>" "<base_url>" <auth_type> "<Category>"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh <platform_id> <action|trigger> <category> <verb> "<Node Name>"
bash .claude/skills/ferroflux-integration/scripts/pre-lint.sh platforms/<platform_id>/
bash .claude/skills/ferroflux-integration/scripts/inventory.sh [platform_id]
```

---

## Process

### Step 0 — Check what already exists (when expanding a platform)

Before adding nodes to an existing platform, always run inventory first:

```bash
bash .claude/skills/ferroflux-integration/scripts/inventory.sh <platform_id>
```

Example output:
```
Slack (slack)
  Base URL: https://slack.com/api

  Actions (4):
    • slack.action.channels.list
    • slack.action.files.upload
    • slack.action.reactions.add
    • slack.action.users.get

  Triggers (6):
    • slack.trigger.messages.new
    ...
```

This prevents duplicating nodes and shows gaps at a glance.

### Step 1 — Gather API information

If the user gave you a docs URL, fetch it. You need:
- The API's base URL
- Authentication method — Bearer token, API key header, OAuth2, or **none** (some APIs like Open-Meteo are completely open)
- Which endpoints to create nodes for

Ask the user if you're unsure which endpoints they want.

**Cloud vs. local/self-hosted variants:** If the integration is a cloud or hosted version of a tool that also has a local or self-hosted version (e.g., Ollama Cloud vs. Ollama local, Supabase Cloud vs. self-hosted Supabase), explicitly verify which endpoints are available in the specific variant being integrated. Do not assume all local endpoints exist in the cloud API — they often don't. Always check the cloud-specific documentation, not just the general API reference.

### Step 2 — Choose the platform ID

Use a short lowercase identifier: `github`, `resend`, `openai`, `stripe`, etc.

### Step 3 — Generate the platform file using the scaffold script

**Run the scaffold script instead of writing the platform file from scratch.** It generates a structurally-valid skeleton and handles all auth patterns correctly.

```bash
# Syntax: scaffold-platform.sh <id> "<Name>" "<base_url>" <auth_type> "<Category>"
# auth_type: bearer | custom:<header-name> | oauth2 | none

bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh stripe "Stripe" "https://api.stripe.com/v1" bearer "E-Commerce"
bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh gemini "Google Gemini" "https://generativelanguage.googleapis.com" "custom:x-goog-api-key" "AI/ML"
bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh google-calendar "Google Calendar" "https://www.googleapis.com/calendar/v3" oauth2 "Calendar"
```

Then open the generated file and:
1. Verify the base_url and auth pattern
2. Add any extra platform-level headers the API requires (e.g., `Accept`, versioning headers)
3. Update the description

File: `platforms/<platform_id>/<platform_id>.yaml`

```yaml
meta:
  id: <platform_id>
  name: <Human Name>
  type: Platform                    # MUST be exactly "Platform"
  category: <Category>              # e.g. Communication, Developer Tools, AI/ML
  description: "Connects to the <Name> API."
  version: "1.0.0"

config:
  base_url: "https://api.example.com"   # No trailing slash
  headers:
    Content-Type: "application/json"
    Authorization: "Bearer PASTE_YOUR_KEY_HERE"   # Only include if the API requires auth

settings:
  - name: api_key
    label: "API Key"
    type: string
    required: true
```

#### Auth Patterns

**Bearer token / API key** (most common):
```yaml
config:
  headers:
    Authorization: "Bearer PASTE_YOUR_KEY_HERE"
settings:
  - name: api_key
    label: "API Key"
    type: string
    required: true
```

**Custom header auth** (e.g., Gemini uses `x-goog-api-key`, Azure OpenAI uses `api-key`):
```yaml
config:
  headers:
    Content-Type: "application/json"
    x-goog-api-key: "PASTE_YOUR_KEY_HERE"
settings:
  - name: api_key
    label: "API Key"
    type: string
    required: true
```

**OAuth2 platforms** (Google APIs, Microsoft Graph, HubSpot, Salesforce, Pipedrive, etc.) — use `connection_select` instead of a raw API key. The engine resolves and auto-refreshes OAuth2 tokens from stored credentials at runtime; no `Authorization` header is needed in the platform file:
```yaml
config:
  base_url: "https://www.googleapis.com/calendar/v3"
  headers:
    Content-Type: "application/json"
    # No Authorization header — injected automatically from the stored connection

settings:
  - name: connection
    label: "Google Account"
    type: connection_select
    required: true
```

The `connection_select` setting type lets users pick from stored connections in the FerroFlux UI. The engine resolves auth before node execution based on the `auth_type` stored with the connection:
- `Bearer` — API key / static token → `Authorization: Bearer <token>`
- `Basic` — username:password base64 → `Authorization: Basic <encoded>`
- `Custom Scheme` — custom scheme + credentials → `Authorization: <scheme> <token>`
- `OAuth2` — full token refresh flow → `Authorization: Bearer <access_token>` (refreshed automatically if expired)

**Auth-free APIs** (e.g., Open-Meteo, public data APIs): Omit `Authorization` from `config.headers` entirely and omit the `api_key` setting. Don't add a placeholder — an empty header just adds noise.

### Step 4 — Generate node skeletons, then fill in the API details

**Run the scaffold script for each node instead of writing YAML from scratch.** The skeleton is structurally valid out of the box — correct ports, returns block, switch pattern, file naming. You fill in only the API-specific parts.

```bash
# Syntax: scaffold-node.sh <platform_id> <action|trigger> <category> <verb> "<Node Name>"

bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh stripe action payments create "Create Payment Intent"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh stripe trigger payments succeeded "Payment Succeeded"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh slack action messages send "Send Message"
```

After running, edit the generated file to:
1. Replace `{{ platform.base_url }}/TODO` with the real path
2. Add the runtime `inputs` the node needs (remove commented-out examples)
3. Add any `settings` fields
4. Fill in the `body` fields for POST/PUT/PATCH
5. Add all valid HTTP success codes (201, 202, 204, etc.)
6. Optionally use `json_query` to extract specific fields before emitting

#### Action Nodes

File: `platforms/<platform_id>/action.<category>.<verb>.yaml`

ID format: `<platform_id>.<category>.<verb>` — e.g. `github.issues.create`, `resend.emails.send`

**Standard action template** (for reference — prefer using scaffold-node.sh):

```yaml
meta:
  id: <platform_id>.<category>.<verb>
  name: <Human Name>
  category: <Category>
  type: Action
  platform: <platform_id>           # Links to the platform file
  description: "<What this node does.>"
  version: "1.0.0"

interface:
  inputs:
    - name: Exec                    # ALWAYS first — triggers execution
      type: flow
    - name: <runtime_param>         # Dynamic values wired from previous nodes
      type: string                  # string | object | array | number | boolean | blob
  outputs:
    - name: Success                 # ALWAYS include both flow ports
      type: flow
    - name: Error
      type: flow
    - name: <data_output>           # Data output(s) after flow ports
      type: object                  # or array, string, etc.
  settings:
    - name: <static_config>         # User-configured in the UI, not wired at runtime
      label: "<Label>"
      type: string                  # string | number | boolean | select | textarea | list | connection_select
      required: true

execution:
  - id: request
    tool: http_client
    params:
      method: POST                  # GET | POST | PUT | DELETE | PATCH
      url: "{{ platform.base_url }}/path/{{ get 'inputs.param' }}"
      headers: "{{ platform.headers }}"
      body:
        field: "{{ get 'inputs.field' }}"
        config: "{{ get 'settings.config' }}"
    returns:
      status: status_code           # ALWAYS map status and body
      body: response_body

  - id: check_status
    tool: switch
    params:
      value: "{{ steps.request.status_code }}"
      cases:
        - condition: "200"          # Add all success codes the API can return
          output: success
        - condition: "201"
          output: success
        - condition: default
          output: error

routing:
  match: "{{ steps.check_status.branch }}"
  cases:
    success:
      - tool: emit
        params:
          port: Success
      - tool: emit
        params:
          port: <data_output>
          value: "{{ steps.request.response_body }}"
    error:
      - tool: emit
        params:
          port: Error
          value: "{{ steps.request.response_body }}"
```

#### Trigger Nodes

File: `platforms/<platform_id>/trigger.<category>.<event>.yaml`

ID format: `<platform_id>.<category>.<event>` — e.g. `slack.messages.new`, `stripe.payments.succeeded`

Key differences from actions:
- `type: Trigger` in meta
- **No `inputs:` array** — triggers have no Exec input
- Only `Success` output for the flow port (no `Error`)
- Use `{{ get 'event.cursor' }}` for stateful polling (tracks the last-seen item across runs)

```yaml
meta:
  id: <platform_id>.<category>.<event>
  name: <Human Name>
  category: <Category>
  type: Trigger
  platform: <platform_id>
  description: "Fires when <event description>."
  version: "1.0.0"

interface:
  outputs:
    - name: Success
      type: flow
    - name: <data_output>
      type: object
  settings:
    - name: poll_interval
      label: "Poll Interval (minutes)"
      type: number
      required: false
      default: 1
      min: 1

execution:
  - id: request
    tool: http_client
    params:
      method: GET
      url: "{{ platform.base_url }}/endpoint"
      headers: "{{ platform.headers }}"
      query:
        oldest: "{{ get 'event.cursor' }}"   # Stateful cursor — tracks last-seen item
        limit: 100
    returns:
      status: status_code
      body: response_body

routing:
  match: "{{ steps.request.status_code }}"
  cases:
    "200":
      - tool: emit
        params:
          port: Success
      - tool: emit
        params:
          port: <data_output>
          value: "{{ steps.request.response_body }}"
```

---

## Advanced Request Capabilities

### Multipart/Form-Data (for file uploads and form submissions)

Use `body_type: multipart` for APIs that require `multipart/form-data`:

```yaml
  - id: upload
    tool: http_client
    params:
      method: POST
      url: "{{ platform.base_url }}/files.upload"
      headers: "{{ platform.headers }}"
      body_type: multipart
      parts:
        - name: file
          content_var: "inputs.file"             # References a blob input port
          filename: "{{ get 'inputs.filename' }}"
          content_type: "application/octet-stream"
        - name: channels
          content: "{{ get 'inputs.channel_id' }}"    # Literal text (templating supported)
        - name: metadata
          content_json: {"key": "value"}         # JSON serialized to string
    returns:
      status: status_code
      body: response_body
```

**Part content sources:**
- `content` — literal string; supports `{{ get '...' }}` templating
- `content_json` — JSON object/array serialized as a string part
- `content_var` — references a context variable by name; if it holds a blob ticket, raw bytes are read from BlobStore; otherwise JSON-serialized

**For file input ports, use `type: blob`:**
```yaml
interface:
  inputs:
    - name: Exec
      type: flow
    - name: file
      type: blob
    - name: filename
      type: string
```

### Binary Request Bodies

For sending raw bytes (audio, images, binary files):

```yaml
  - id: transcribe
    tool: http_client
    params:
      method: POST
      url: "{{ platform.base_url }}/audio/transcriptions"
      headers: "{{ platform.headers }}"
      body_type: binary
      body_var: "inputs.audio"        # Blob input variable
      content_type: "audio/mpeg"
    returns:
      status: status_code
      body: response_body
```

### Streaming Responses (LLM APIs)

For streaming chat completions and similar SSE-based LLM endpoints:

```yaml
  - id: stream
    tool: http_client
    params:
      method: POST
      url: "{{ platform.base_url }}/chat/completions"
      headers: "{{ platform.headers }}"
      stream: true
      step_id: "llm_stream"           # Optional — used for event correlation in UI
      body:
        model: "{{ get 'settings.model' }}"
        messages: "{{ inputs.messages }}"
        stream: true
    returns:
      status: status_code
      body: response_body             # Returns { chunks: [...], text: "full text", total_chunks: N }
```

**Auto-detected streaming formats:**
- OpenAI / Azure OpenAI: `choices[0].delta.content`
- Anthropic Claude: `delta.text`
- Google Gemini: `candidates[0].content.parts[0].text`

**Note:** Inbound SSE subscription triggers (`trigger.sse`) are not yet implemented in the runtime. If a platform provides push webhooks or SSE-based triggers, document those nodes in `GAPS.md` with reference to GAP-005.

### Pagination

Use the `paginate` tool for list endpoints that may return multiple pages:

```yaml
  - id: list_all
    tool: paginate
    params:
      url: "{{ platform.base_url }}/items"
      method: GET
      headers: "{{ platform.headers }}"
      strategy: cursor          # cursor | page_token | offset | link_header
      cursor_param: "cursor"    # query param name for cursor-based pagination
      per_page: 100
      max_pages: 50
    returns:
      items: all_items
      total_pages: page_count
      total_items: item_count
```

**Pagination strategies:**
- `cursor` — reads next cursor from response JSON; passes via `cursor_param` query param
- `page_token` — Google-style `pageToken` / `nextPageToken` param
- `offset` — classic `offset` + `limit` query params
- `link_header` — follows `Link: <url>; rel="next"` response header

### Webhook Signature Verification

For triggers that receive inbound webhooks with HMAC signatures:

```yaml
  - id: verify
    tool: verify_signature
    params:
      body: "{{ event.raw_body }}"
      secret: "{{ get 'settings.webhook_secret' }}"
      signature: "{{ event.headers['X-Hub-Signature-256'] }}"
      algorithm: "hmac-sha256"    # or hmac-sha1
      encoding: "hex"             # or base64
    returns:
      valid: is_valid

  - id: check_valid
    tool: switch
    params:
      value: "{{ steps.verify.is_valid }}"
      cases:
        - condition: "true"
          output: verified
        - condition: default
          output: invalid
```

Auto-strips platform prefixes (`sha256=`, `v0=`, `sha1=`, etc.). Uses constant-time comparison to prevent timing attacks.

---

## Inputs vs Settings — When to Use Each

| Use `inputs`                                                        | Use `settings`                                 |
| ------------------------------------------------------------------- | ---------------------------------------------- |
| Data that varies per execution and should be wired from other nodes | Static config the user sets once in the UI     |
| e.g., `to` (email recipient), `owner` (repo owner), `body`          | e.g., `from` address, `per_page`, `sort` order |

---

## Templating Rules

- **Always use `{{ get 'path' }}`** when mixing variables into strings:
  ```yaml
  url: "{{ platform.base_url }}/users/{{ get 'inputs.user_id' }}/repos"
  ```
- For isolated single-value params, bare `{{ }}` is fine:
  ```yaml
  headers: "{{ platform.headers }}"
  value: "{{ steps.request.response_body }}"
  ```
- Settings access: always use `{{ get 'settings.field_name' }}` — the shorthand `{{ settings.x }}` is NOT supported
- Input access: `{{ get 'inputs.field_name' }}`

---

## Platform Object Reference

The `platform` object exposes **exactly two properties** in node execution:

- `{{ platform.base_url }}` — the configured base URL from the platform file
- `{{ platform.headers }}` — the full merged headers object, including auth credentials

**Never** reference `platform.api_key`, `platform.token`, `platform.secret`, or any other property — they do not exist. Auth credentials are baked into `platform.headers` at runtime; accessing them individually is not possible.

---

## Data Outputs

Don't emit the raw response object when users only need a specific field. Use `json_query` to extract exactly what's useful and declare the correct type on the output port.

**`json_query` tool:**
```yaml
  - id: extract
    tool: json_query
    params:
      json: "{{ steps.request.response_body }}"
      path: "/data/items"         # JSONPointer path (leading slash required)
    returns:
      result: items_array
```

**Pattern:**
```yaml
outputs:
  - name: embedding
    type: array          # matches what's actually emitted — not "object"

routing:
  cases:
    success:
      - tool: json_query
        params:
          json: "{{ steps.request.response_body }}"
          path: "/embedding/values"      # extract the array, not the whole response
        returns:
          result: embedding_values
      - tool: emit
        params:
          port: embedding
          value: "{{ embedding_values }}"
```

**Rules:**
- Match the output `type` to what is actually emitted: `array` for arrays, `string` for strings, `object` only when the whole object is genuinely useful.
- A raw `response: object` output is acceptable as a secondary "escape hatch" port, but primary data ports should emit extracted values.
- **If a node accepts `tools` as an input**, it must include a `tool_calls` output port (`type: array`) with a `json_query` step extracting the tool call results. This lets users wire tool responses directly without manually parsing the response object.

---

## Setting Field Types

| Type                | Use for                                    |
| ------------------- | ------------------------------------------ |
| `string`            | Text input — API keys, IDs, names          |
| `number`            | Numeric values — limits, timeouts, counts  |
| `boolean`           | On/off flags                               |
| `select`            | Dropdown with fixed choices                |
| `textarea`          | Multi-line text — prompts, templates       |
| `list`              | Array of items — labels, tags, field lists |
| `connection_select` | OAuth2 / stored connection picker          |

**`select` with static options:**
```yaml
settings:
  - name: model
    label: "Model"
    type: select
    required: true
    default: "gpt-4o"
    options:
      - value: "gpt-4o"
        label: "GPT-4o"
      - value: "gpt-4o-mini"
        label: "GPT-4o Mini"
```

**`select` with dynamic options provider:**
```yaml
    options_provider: "openai.models.list"  # Node ID that returns available options
```

**Conditional visibility:**
```yaml
    show_if: "settings.advanced_mode == true"
```

---

## No Stubs Policy

If an endpoint cannot be fully implemented with the available tools, omit it entirely. Never commit a broken or incomplete node — no placeholder comments, no half-wired bodies. A missing node is better than one that silently fails or misleads users.

**Currently resolved capabilities** (no longer a reason to omit nodes):
- GAP-002: Multipart/form-data file upload — ✅ use `body_type: multipart`
- GAP-003: Pagination across multiple pages — ✅ use `paginate` tool
- GAP-004: Webhook signature verification — ✅ use `verify_signature` tool
- GAP-006: Binary request bodies — ✅ use `body_type: binary`

**Still open — omit and document in GAPS.md:**
- GAP-005: Inbound SSE subscription triggers (outbound SSE streaming via `stream: true` is fine; it's the inbound trigger that isn't yet implemented)

**Document gaps in `GAPS.md`:** Whenever you omit a node due to a system limitation, add an entry to `platforms/<platform_id>/GAPS.md`. Cross-reference `SYSTEM_GAPS.md` at the repo root — reference the gap ID in the entry rather than re-explaining it. Use this format:

```markdown
# <Platform Name> — Integration Gaps

## <Node Name> (`<endpoint>`)

- **Why omitted:** <What system capability is missing>
- **API endpoint:** `<METHOD> <full endpoint URL>`
- **Docs:** <link to relevant API docs section>
- **Value:** <High / Medium / Low> — <one sentence on how commonly users would need this>
- **Unblocked by:** <What system change would enable this>
```

If there are no omissions, do not create `GAPS.md`.

### Step 5 — Validate your work

Run validation in two stages. The pre-lint is fast (bash grep) and catches the class of template bug the Rust validator cannot see. The cargo validator catches structural errors.

**Stage 1 — pre-lint (fast, catches template bugs):**
```bash
bash .claude/skills/ferroflux-integration/scripts/pre-lint.sh platforms/<platform_id>/
```

The pre-lint catches:
- `bare-settings-access` — `{{ settings.x }}` without `get` (silent runtime bug, NOT caught by Rust validator)
- `bare-inputs-access` — `{{ inputs.x }}` without `get` in string interpolation (same)
- `missing-returns` — `http_client` step with no `returns:` block
- `hardcoded-url` — URL not using `{{ platform.base_url }}`
- `trailing-slash` — `base_url` with trailing slash
- `unfilled-todo` — `/TODO` placeholder left in URL
- `invalid-platform-property` — `platform.api_key` / `platform.token` (don't exist)

Fix all **errors** before proceeding. Warnings are informational.

**Stage 2 — cargo validator (structural correctness):**
```bash
cargo run --bin ferroflux-validate -- platforms/<platform_id>/
```

Fix any **errors** before finishing. **Warnings** (e.g., hardcoded URLs for APIs with multiple subdomains) are informational — use your judgment. A clean run looks like:

```
platforms/resend/resend.yaml               OK
platforms/resend/action.emails.send.yaml   OK
platforms/resend/action.domains.list.yaml  OK

3 files checked, 0 errors, 0 warnings
```

---

## Checklist Before Finishing

> Run `bash .claude/skills/ferroflux-integration/scripts/pre-lint.sh platforms/<platform_id>/` before checking these off — it catches the most common template bugs automatically.

- [ ] Platform `meta.type` is exactly `Platform`
- [ ] Auth pattern matches the API type (Bearer, custom header, OAuth2 `connection_select`, or none)
- [ ] If API is auth-free: no `Authorization` header in `config.headers`
- [ ] `config.base_url` has no trailing slash
- [ ] Every **action** node has `Exec` as first input (type: flow)
- [ ] Every **trigger** node has NO inputs (triggers have no Exec)
- [ ] Every action node has both `Success` and `Error` outputs (type: flow)
- [ ] Every trigger node has `Success` output (no `Error` needed)
- [ ] Every `http_client` step has a `returns:` block with `status` and `body`
- [ ] Node URLs use `{{ platform.base_url }}` — no hardcoded base URLs
- [ ] Node `meta.platform` matches the platform's `meta.id`
- [ ] Node IDs follow `platform.category.verb` naming
- [ ] Action files saved to `platforms/<platform_id>/action.<category>.<verb>.yaml`
- [ ] Trigger files saved to `platforms/<platform_id>/trigger.<category>.<event>.yaml`
- [ ] Every setting declared in `interface.settings` is referenced in the `execution` block
- [ ] Settings are accessed as `{{ get 'settings.field_name' }}` — NOT `{{ settings.x }}`
- [ ] Every data output port type matches what is actually emitted (`array` → `array`, `string` → `string`)
- [ ] Every node that accepts a `tools` input has a `tool_calls` output port (`type: array`) with a `json_query` extraction step
- [ ] File upload nodes use `body_type: multipart` (not skipped as "unsupported")
- [ ] List endpoints that may paginate use the `paginate` tool
- [ ] No broken stubs — if an endpoint can't be implemented cleanly, it is omitted entirely
- [ ] If any nodes were omitted, `platforms/<platform_id>/GAPS.md` exists and documents each omission
- [ ] `cargo run --bin ferroflux-validate -- platforms/<platform_id>/` passes with 0 errors

---

## Complete Example — GitHub Create Issue

**`platforms/github/github.yaml`:**
```yaml
meta:
  id: github
  name: GitHub
  type: Platform
  category: Developer Tools
  description: "Connects to the GitHub REST API."
  version: "1.0.0"

config:
  base_url: "https://api.github.com"
  headers:
    Content-Type: "application/json"
    Accept: "application/vnd.github+json"
    X-GitHub-Api-Version: "2022-11-28"
    Authorization: "Bearer PASTE_YOUR_TOKEN_HERE"

settings:
  - name: api_key
    label: "Personal Access Token"
    type: string
    required: true
```

**`platforms/github/action.issues.create.yaml`:**
```yaml
meta:
  id: github.issues.create
  name: Create Issue
  category: Issues
  type: Action
  platform: github
  description: "Create a new issue in a GitHub repository."
  version: "1.0.0"

interface:
  inputs:
    - name: Exec
      type: flow
    - name: owner
      type: string
    - name: repo
      type: string
    - name: title
      type: string
    - name: body
      type: string
  outputs:
    - name: Success
      type: flow
    - name: Error
      type: flow
    - name: issue
      type: object
  settings:
    - name: labels
      label: "Labels (comma-separated)"
      type: string
      required: false
      placeholder: "bug,enhancement"

execution:
  - id: request
    tool: http_client
    params:
      method: POST
      url: "{{ platform.base_url }}/repos/{{ get 'inputs.owner' }}/{{ get 'inputs.repo' }}/issues"
      headers: "{{ platform.headers }}"
      body:
        title: "{{ get 'inputs.title' }}"
        body: "{{ get 'inputs.body' }}"
        labels: "{{ get 'settings.labels' }}"
    returns:
      status: status_code
      body: response_body

  - id: check_status
    tool: switch
    params:
      value: "{{ steps.request.status_code }}"
      cases:
        - condition: "201"
          output: success
        - condition: default
          output: error

routing:
  match: "{{ steps.check_status.branch }}"
  cases:
    success:
      - tool: emit
        params:
          port: Success
      - tool: emit
        params:
          port: issue
          value: "{{ steps.request.response_body }}"
    error:
      - tool: emit
        params:
          port: Error
          value: "{{ steps.request.response_body }}"
```
