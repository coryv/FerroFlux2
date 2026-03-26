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
- Authentication method (usually Bearer token or API key header)
- Which endpoints to create nodes for

Ask the user if you're unsure which endpoints they want.

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
    Authorization: "Bearer PASTE_YOUR_KEY_HERE"   # MUST be in config.headers, not under auth:

settings:
  - name: api_key
    label: "API Key"
    type: string
    required: true
```

**Critical:** Auth MUST go in `config.headers.Authorization`. Do not use an `auth:` key — it won't be picked up by nodes.

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
- Settings access: `{{ get 'settings.field_name' }}` or `{{ settings.field_name }}`
- Input access: `{{ get 'inputs.field_name' }}`

---

## Checklist Before Finishing

- [ ] Platform `meta.type` is exactly `Platform`
- [ ] `config.headers` has `Authorization` entry (not under `auth:`)
- [ ] `config.base_url` has no trailing slash
- [ ] Every node has `Exec` as first input (type: flow)
- [ ] Every node has both `Success` and `Error` outputs (type: flow)
- [ ] Every `http_client` step has a `returns:` block with `status` and `body`
- [ ] Node URLs use `{{ platform.base_url }}` — no hardcoded base URLs
- [ ] Node `meta.platform` matches the platform's `meta.id`
- [ ] Node IDs follow `platform.category.verb` naming
- [ ] File saved to `platforms/<platform_id>/action.<category>.<verb>.yaml`

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
