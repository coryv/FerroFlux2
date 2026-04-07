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

## Scripts & Tools

Helper scripts live in `.claude/skills/ferroflux-integration/scripts/`. **Always run these from the FerroFlux2 project root.**

| Script                 | When to use                                                                                         |
| ---------------------- | --------------------------------------------------------------------------------------------------- |
| `scaffold-platform.sh` | Creating a new platform — generates a structurally-valid `<id>.yaml` skeleton                       |
| `scaffold-node.sh`     | Creating each node — generates a structurally-valid action or trigger YAML skeleton                 |
| `pre-lint.sh`          | After editing any YAML — catches leftover Handlebars `{{ }}` syntax and other common bugs            |
| `inventory.sh`         | Before expanding an existing platform — shows what nodes already exist                              |

### Security Tools
FerroFlux requires all integrations to declare permissions and be cryptographically signed for production use.

| Tool             | Command                                                                 | Purpose                                                                 |
| ---------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `ferroflux-sign` | `cargo run -p ferroflux-integration --bin ferroflux-sign -- -f <file>` | Signs an integration YAML file with a private key.                      |
| `validate`       | `cargo run --bin ferroflux-validate -- <dir>`                           | Audits structure, permissions, and signatures.                          |

---

## Process

### Step 0 — Check what already exists
Before adding nodes to an existing platform, always run inventory first:
```bash
bash .claude/skills/ferroflux-integration/scripts/inventory.sh <platform_id>
```

### Step 1 — Gather API information
Fetch the docs URL. You need the Base URL, Auth method, and specific endpoints.

### Step 2 — Choose the platform ID
Use a short lowercase identifier: `github`, `stripe`, `aws`, etc.

### Step 3 — Generate the platform file
**Run the scaffold script instead of writing from scratch.**
```bash
bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh <id> "<Name>" "<base_url>" <auth_type> "<Category>"
```

### Step 4 — Generate node skeletons
**Run the scaffold script for each node.**
```bash
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh <platform_id> <action|trigger> <category> <verb> "<Node Name>"
```

### Step 5 — Declare Permissions (Mandatory)
Every integration MUST declare the network domains it intended to access in the `meta` block. The engine blocks any request to a domain not in this list.

File: `platforms/<id>/<id>.yaml` (and all node files)
```yaml
meta:
  id: myplatform
  permissions:
    - "network:api.myplatform.com"
    - "network:*.myplatform.static.com"
```

### Step 6 — Validate and Lint
1. **Pre-lint:** `bash .claude/skills/ferroflux-integration/scripts/pre-lint.sh platforms/<platform_id>/`
2. **Rust Validate:** `cargo run --bin ferroflux-validate -- platforms/<platform_id>/`

### Step 7 — Sign the Integration (Final Step)
Once validation passes, sign the files using the `ferroflux-sign` tool. This ensures integrity and authorship.
```bash
# Requires FERROFLUX_PRIVATE_KEY environment variable
cargo run -p ferroflux-integration --bin ferroflux-sign -- -f platforms/<id>/<id>.yaml
cargo run -p ferroflux-integration --bin ferroflux-sign -- -f platforms/<id>/action.<name>.yaml
```

---

## Security & Verification

### Permission Model
- **Network Permissions:** `network:<domain>` (e.g. `network:api.slack.com`). Supports wildcards for subdomains: `network:*.amazonaws.com`.
- **Database/Core:** Nodes using `sql_query` or `mongo_query` do not need network permissions as they use internal drivers.
- **Enforcement:** Any `http_client` call to a domain not in the meta's `permissions` list will trigger a runtime error and a validation failure.

### Signing & Key Rotation
- **Signatures:** Signatures are stored in `meta.signature`.
- **Rotation:** FerroFlux supports versioned keys (v1, v2). When rotating keys, update the `version` flag in the sign tool:
  `ferroflux-sign --file <path> --key <hex> --version v2`
- **Trust:** Official integrations are signed with the FerroFlux root key. Community integrations signed with unknown keys will display a warning.

---

## CEL Expressions

FerroFlux uses [CEL (Common Expression Language)](https://cel.dev) for all variable references in YAML. There is no Handlebars/`{{ }}` syntax.

**The `=` prefix is mandatory.** A string without `=` is always a literal — the engine never guesses. Any expression without `=` will be passed as a raw string, which is almost always a bug.

### Variable Access
```yaml
headers: =platform.headers              # pass object directly
body:
  path: =inputs.path                    # runtime input
  mode: =self.settings.mode             # node setting (resolved)
  result: =steps.request.response_body  # previous step output
```

For URL construction (string concatenation), use a folded block scalar so the `=` prefix is unambiguous:
```yaml
url: >-
  =platform.base_url + "/v1/users/" + inputs.user_id
```

Or a quoted string for shorter expressions:
```yaml
url: "=platform.base_url + \"/v1/users\""
```

### CEL Operators
```yaml
# Ternary
url: >-
  =platform.base_url + (self.settings.version == "v2" ? "/v2" : "/v1") + "/endpoint"

# String concat (quoted form)
header_val: "=\"Bearer \" + platform.token"

# Comparison and logic
value: "=inputs.count > 0 && self.settings.enabled"

# Optional field guard
timeout: "=has(self.settings.timeout) ? self.settings.timeout : 30"
```

### `json()` Function
Serialize a value to a JSON string (useful in header values):
```yaml
headers:
  X-API-Arg: '=json({"path": inputs.path, "mode": self.settings.mode})'
```

### Plain Literals
Strings without `=` are always treated as literals — no CEL evaluation:
```yaml
method: POST            # literal — no quotes needed
Content-Type: application/octet-stream
```

### Accessing Platform Config
`platform` (singular) refers to the **active platform's config** — the keys defined in the platform YAML `config:` block plus a `headers` key built from `auth`. Use `self.settings` for the current node's resolved settings:
```yaml
url: >-
  =platform.base_url + "/endpoint"
headers: =platform.headers              # pre-built auth headers
body:
  name: =self.settings.name             # node's own setting
  value: =inputs.value                  # data from upstream edge
```

---

## Inputs vs Settings — When to Use Each

| Use `inputs`                                                        | Use `settings`                                 |
| ------------------------------------------------------------------- | ---------------------------------------------- |
| Data that varies per execution and should be wired from other nodes | Static config the user sets once in the UI     |
| e.g., `to` (email recipient), `owner` (repo owner), `body`          | e.g., `from` address, `per_page`, `sort` order |

---

## Checklist Before Finishing

- [ ] `meta.permissions` contains all domains accessed via `http_client`
- [ ] `config.base_url` has no trailing slash
- [ ] Every **action** node has `Exec` as first input (type: flow)
- [ ] Every action node has both `Success` and `Error` outputs (type: flow)
- [ ] Every `http_client` step has a `returns:` block with `status` and `body`
- [ ] Node URLs use `platform.base_url + "/path"` where possible
- [ ] Settings accessed as `settings.field_name`, inputs as `inputs.field_name`
- [ ] `cargo run --bin ferroflux-validate` passes with 0 errors
- [ ] All files have been signed using `ferroflux-sign` (if a key is available)
