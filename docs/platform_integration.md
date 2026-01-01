# Platform Integration Guide

**Platforms** represent shared services or configurations (like an API provider) that multiple Nodes can use. They provide a central place to manage base URLs, authentication, and global headers.

## Directory Structure
Platform definitions live in `platforms/<platform_id>/`.
The entry point is a `.yaml` file (commonly named after the platform or `credentials.yaml`).

Example:
```
platforms/
  core/
    trigger.scheduler.yaml
    action.http.yaml
  openai/
    openai.yaml      # Platform Definition
    chat.yaml        # Node Definition
    image.yaml       # Node Definition
```

---

## Platform Definition (`openai.yaml`)

This file defines the shared configuration.

```yaml
meta:
  id: openai                # Platform ID
  name: OpenAI
  category: AI/ML
  type: Platform            # MUST be "Platform"
  description: "Connects to OpenAI API"
  version: "1.0.0"

config:                     # Shared configuration variables
  base_url: "https://api.openai.com/v1"
  headers:
    Content-Type: "application/json"
    Authorization: "Bearer PASTE_YOUR_KEY_HERE" # See Secrets note below

settings:                   # (Optional) Global settings for the platform
  - name: organization_id
    label: "Org ID"
    type: string
```

### Note on Secrets
> [!NOTE]
> The `secrets` namespace (e.g., `{{ secrets.MY_KEY }}`) is currently under active development. For now, sensitive values should be provided either directly in the `config` block or via environment variables handled by the host application.

---

## Using a Platform in a Node

In your Node Definition (`.yaml`), specify the `platform` in the metadata:

```yaml
meta:
  id: openai.chat
  platform: openai          # Links this node to the 'openai' platform
  # ...
```

When this node executes, the Platform's `config` is automatically injected into the context under the `platform` namespace.

**Example Usage:**
```yaml
execution:
  - id: req
    tool: http_client
    params:
      method: POST
      url: "{{ platform.base_url }}/chat/completions"  # Uses config.base_url
      headers: "{{ platform.headers }}"                 # Uses config.headers
```
