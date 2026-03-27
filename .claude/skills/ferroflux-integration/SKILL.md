---
name: ferroflux-integration
description: Build a FerroFlux platform integration from an API. Use this skill any time the user wants to add a new integration, connector, or platform to FerroFlux — including when they say "add integration", "create a platform", "build a connector", "integrate with X", "add X support", or provide an API docs URL and ask you to build nodes for it. Always use this skill when creating or modifying YAML files under the platforms/ directory.
---

# FerroFlux Integration Builder

Use this skill to create a complete FerroFlux integration for a third-party API.

## What You're Building

Every integration consists of two types of files saved under `platforms/<platform_id>/`:

1. **Platform file** (`<platform_id>.yaml`) — shared config (base URL, auth headers)
2. **Node files** (`action.<category>.<verb>.yaml`) — one per API operation

---

## Process

### Step 1 — Gather API information

If the user gave you a docs URL, fetch it. You need:
- The API's base URL
- Authentication method — Bearer token, API key header, or **none** (some APIs like Open-Meteo are completely open)
- Which endpoints to create nodes for

Ask the user if you're unsure which endpoints they want.

**Cloud vs. local/self-hosted variants:** If the integration is a cloud or hosted version of a tool that also has a local or self-hosted version (e.g., Ollama Cloud vs. Ollama local, Supabase Cloud vs. self-hosted Supabase), explicitly verify which endpoints are available in the specific variant being integrated. Do not assume all local endpoints exist in the cloud API — they often don't. Always check the cloud-specific documentation, not just the general API reference.

### Step 2 — Choose the platform ID

Use a short lowercase identifier: `github`, `resend`, `openai`, `stripe`, etc.

### Step 3 — Write the platform file

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

**Auth-required APIs:** Put auth in `config.headers.Authorization` (Bearer token or API key). Do not use an `auth:` key — it won't be picked up by nodes. Add an `api_key` setting so the user knows what to configure.

**Auth-free APIs** (e.g., Open-Meteo, public data APIs): Omit `Authorization` from `config.headers` entirely and omit the `api_key` setting. Don't add a placeholder — an empty header just adds noise.

### Step 4 — Write node files

File: `platforms/<platform_id>/action.<category>.<verb>.yaml`

ID format: `<platform_id>.<category>.<verb>` — e.g. `github.issues.create`, `resend.emails.send`

**Standard node template:**

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
      type: string                  # string | object | array | number | boolean
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
      type: string                  # string | number | boolean | select | textarea
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

**No stubs:** If an endpoint cannot be fully implemented with the available tools (e.g., multipart file upload requiring custom binary framing that `http_client` doesn't support), omit it entirely. Never commit a broken or incomplete node — no placeholder comments, no half-wired bodies. A missing node is better than one that silently fails or misleads users.

**Document gaps in `GAPS.md`:** Whenever you omit a node due to a system limitation, add an entry to `platforms/<platform_id>/GAPS.md`. Cross-reference `SYSTEM_GAPS.md` at the repo root — if the limitation is already tracked there (e.g., GAP-002 for multipart upload), reference the gap ID in the entry rather than re-explaining it. This file is reviewed by the FerroFlux team to prioritize system improvements — a single gap that blocks 10 integrations is worth fixing at the platform level. Use this format:

```markdown
# <Platform Name> — Integration Gaps

## <Node Name> (`<endpoint>`)

- **Why omitted:** <What system capability is missing — e.g., "`http_client` does not support multipart/form-data bodies required for binary file upload">
- **API endpoint:** `<METHOD> <full endpoint URL>`
- **Docs:** <link to relevant API docs section>
- **Value:** <High / Medium / Low> — <one sentence on how commonly users would need this>
- **Unblocked by:** <What system change would enable this — e.g., "Adding `multipart` body support to `http_client`">
```

If there are no omissions, do not create `GAPS.md`.

### Step 5 — Validate your work

After writing the files, run the FerroFlux validator to catch structural errors before the user hits them at runtime:

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

## Inputs vs Settings — When to Use Each

| Use `inputs` | Use `settings` |
|---|---|
| Data that varies per execution and should be wired from other nodes | Static config the user sets once in the UI |
| e.g., `to` (email recipient), `owner` (repo owner), `body` | e.g., `from` address, `per_page`, `sort` order |

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

## Checklist Before Finishing

- [ ] Platform `meta.type` is exactly `Platform`
- [ ] If API requires auth: `config.headers.Authorization` is set (not under `auth:`)
- [ ] If API is auth-free: no `Authorization` header in `config.headers`
- [ ] `config.base_url` has no trailing slash
- [ ] Every node has `Exec` as first input (type: flow)
- [ ] Every node has both `Success` and `Error` outputs (type: flow)
- [ ] Every `http_client` step has a `returns:` block with `status` and `body`
- [ ] Node URLs use `{{ platform.base_url }}` — no hardcoded base URLs
- [ ] Node `meta.platform` matches the platform's `meta.id`
- [ ] Node IDs follow `platform.category.verb` naming
- [ ] File saved to `platforms/<platform_id>/action.<category>.<verb>.yaml`
- [ ] Every setting declared in `interface.settings` is referenced in the `execution` block
- [ ] Every data output port type matches what is actually emitted (`array` → `array`, `string` → `string` — not everything as `object`)
- [ ] Every node that accepts a `tools` input has a `tool_calls` output port (`type: array`) with a `json_query` extraction step
- [ ] No broken stubs — if an endpoint can't be implemented cleanly, it is omitted entirely
- [ ] If any nodes were omitted, `platforms/<platform_id>/GAPS.md` exists and documents each omission with why, the endpoint, value, and what system change would unblock it
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
        labels: "{{ settings.labels }}"
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
