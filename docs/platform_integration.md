# Platform Integration Guide

**Platforms** represent shared services or configurations (like an API provider) that multiple Nodes can use. They provide a central place to manage base URLs, authentication, and global headers.

## Directory Structure
Platform definitions live in `platforms/<platform_id>/`.
The entry point is typically `credentials.yaml`.

Example:
```
ferroflux/
  platforms/
    openai/
      credentials.yaml
      chat.completion.yaml
      image.generate.yaml
```

---

## Platform Definition (`credentials.yaml`)

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
    Authorization: "Bearer {{ secrets.OPENAI_API_KEY }}"
```

### Accessing Secrets
Use `{{ secrets.VAR_NAME }}` to securely access environment variables or secrets stored in the secret manager. These are resolved at runtime.

---

## Using a Platform in a Node

In your Node Definition (`.yaml`), specify the `platform` in the metadata:

```yaml
meta:
  id: openai.chat
  platform: openai          # Links this node to the 'openai' platform
  # ...
```

When this node executes, the Platform's `config` is injected into the context under the `platform` namespace.

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
