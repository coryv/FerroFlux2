# FerroFlux System Gaps

This file tracks execution engine capabilities that are missing or insufficient to fully implement high-value integration nodes. Each entry includes the affected integrations, what specifically is blocked, and what system change would unblock it.

Per-integration omissions (individual nodes that couldn't be built) are documented in `platforms/<platform_id>/GAPS.md`. This file is for **cross-cutting system-level gaps** that affect multiple integrations simultaneously.

---

## GAP-001 — OAuth2 Token Refresh

**Priority:** Critical
**Status:** Open

### What's missing
The platform credential system only supports static tokens (e.g., a hardcoded `Bearer xoxb-...` in `config.headers`). OAuth2 access tokens expire — typically after 1 hour. There is no mechanism for the runtime to automatically refresh an expired token using a refresh token before executing a node.

### What breaks without it
Every integration that uses OAuth2 silently fails in production once the access token expires. Users must manually rotate tokens or re-authenticate, making workflows unreliable for anything longer-lived than a single session.

### Affected integrations (partial list)
Gmail, Google Drive, Google Sheets, Google Calendar, Google Docs, Slack, Microsoft Teams, Outlook, OneDrive, HubSpot, Salesforce, Dropbox, Notion, Airtable, Pipedrive, Intercom, Zendesk, QuickBooks, Xero, Webflow, Calendly, and most other Tier 1 SaaS platforms.

### What's needed
- A credential store that persists `access_token`, `refresh_token`, `expires_at`, and the token refresh endpoint per platform
- Runtime logic to check `expires_at` before node execution and refresh if expired
- Platform YAML support for declaring the token refresh endpoint and grant parameters

### Notes
This is the single highest-leverage system capability missing. It blocks ~30+ Tier 1 integrations from being production-grade regardless of how well the nodes are built.

---

## GAP-002 — Multipart/Form-Data Request Bodies

**Priority:** High
**Status:** Open

### What's missing
`http_client` only supports JSON request bodies (`Content-Type: application/json`). It cannot construct `multipart/form-data` requests required for file and binary uploads.

### What breaks without it
All file upload nodes must be omitted from integrations. Users cannot upload files to any platform through FerroFlux workflows.

### Affected integrations (partial list)
Slack (Upload File), Google Drive (Upload File), Dropbox (Upload File), AWS S3 (Upload Object), GitHub (Upload Release Asset), Gemini File API (Upload File), AssemblyAI (Upload Audio), ElevenLabs (Upload Voice), Cloudinary (Upload Image/Video), OneDrive (Upload File).

### What's needed
- `http_client` support for `body_type: multipart` with named parts, file references, and per-part `Content-Type` headers
- Ability to reference a binary value from a previous node's output as a part body

---

## GAP-003 — Pagination / Cursor Iteration

**Priority:** High
**Status:** Open

### What's missing
There is no built-in mechanism to follow pagination across multiple API calls. A node makes a single request and emits the first page of results. Subsequent pages (via `next_cursor`, `next_page_token`, `Link: rel="next"` headers, `offset`+`limit`, etc.) cannot be fetched automatically.

### What breaks without it
All "list" and "search" nodes are effectively limited to the first page of results (typically 20–200 items). Workflows that need to process all records — e.g., sync all HubSpot contacts, export all Notion database items — silently return partial data.

### Affected integrations (partial list)
Notion (Query Database), Airtable (List Records), HubSpot (List Contacts/Deals), Salesforce (SOQL Query), Slack (List Channels, List Users, Conversations History), Google Drive (List Files), GitHub (List Issues/PRs/Commits), Jira (Search Issues), Linear (List Issues), Stripe (List Customers/Charges), Shopify (List Orders/Products), and all other paginated list endpoints.

### What's needed
One of:
- A `paginate` execution tool that accepts a request config and a pagination strategy (`cursor`, `page_token`, `offset`, `link_header`) and iterates until exhausted, emitting a merged array
- A loop construct in the execution engine that can re-run a step with an updated parameter until a termination condition is met

---

## GAP-004 — Webhook Signature Verification

**Priority:** High
**Status:** Open

### What's missing
The core `trigger.webhook` node accepts any inbound HTTP request without verifying its authenticity. There is no built-in step for HMAC-SHA256 (or similar) signature verification against a shared secret.

### What breaks without it
Webhook-triggered workflows are insecure — any party that knows the webhook URL can trigger them. Platform-specific verification (Stripe's `Stripe-Signature`, GitHub's `X-Hub-Signature-256`, Slack's `X-Slack-Signature`, Shopify's `X-Shopify-Hmac-Sha256`) must be either skipped or re-implemented ad hoc in each integration.

### Affected integrations (partial list)
Stripe, GitHub, Slack, Shopify, HubSpot, Linear, Twilio, Intercom, WooCommerce, PagerDuty, and any platform that signs outbound webhook payloads.

### What's needed
- A `verify_signature` execution tool (or a setting on `trigger.webhook`) that takes a raw body, a secret, an algorithm (`hmac-sha256`, `hmac-sha1`), and an expected signature header — and routes to Error if verification fails
- Optionally, platform-level declarations of the signature scheme so trigger nodes can self-configure

---

## GAP-005 — Streaming / Server-Sent Events (SSE)

**Priority:** Medium-High
**Status:** Open

### What's missing
`http_client` waits for the full response body before returning. It cannot handle streaming responses (`Transfer-Encoding: chunked`, `Content-Type: text/event-stream`) where tokens or events arrive incrementally.

### What breaks without it
AI completion nodes block until the full response is received, which for long outputs can be many seconds with no feedback. Streaming is the expected UX for LLM integrations. Real-time event feeds (SSE-based triggers) also cannot be implemented.

### Affected integrations (partial list)
OpenAI (Streaming Chat Completion), Anthropic Claude (Streaming Messages), Google Gemini (Streaming Generate), Azure OpenAI (Streaming), Mistral, Groq, Cohere — and the SSE trigger node in the roadmap.

### What's needed
- `http_client` support for `stream: true` that emits partial results incrementally as a node output stream
- A `trigger.sse` node type that connects to an SSE endpoint and fires the workflow on each event

---

## GAP-006 — Binary / Raw Request Bodies

**Priority:** Medium
**Status:** Open

### What's missing
`http_client` cannot send raw binary bodies (e.g., `Content-Type: audio/mpeg`, `image/png`, `application/octet-stream`). Only JSON bodies are supported.

### What breaks without it
APIs that accept raw binary input — rather than base64-encoded JSON or multipart — cannot be called directly.

### Affected integrations (partial list)
ElevenLabs (Send Audio for Speech-to-Speech), AssemblyAI (Upload raw audio), Stability AI (Send image for img2img), Replicate (binary media inputs), Deepgram (audio upload).

### What's needed
- `http_client` support for `body_type: binary` with a `content_type` field and a binary value reference from a prior step output

---

## Summary

| ID | Gap | Priority | Integrations affected |
|---|---|---|---|
| GAP-001 | OAuth2 token refresh | Critical | ~30+ |
| GAP-002 | Multipart/form-data bodies | High | ~10+ |
| GAP-003 | Pagination / cursor iteration | High | ~20+ |
| GAP-004 | Webhook signature verification | High | ~10+ |
| GAP-005 | Streaming / SSE | Medium-High | ~6+ AI + real-time feeds |
| GAP-006 | Binary raw bodies | Medium | ~5+ |

**Recommended build order:** GAP-001 → GAP-003 → GAP-002 → GAP-004 → GAP-006 → GAP-005

GAP-001 (OAuth2) and GAP-003 (pagination) together make the most existing and planned integrations production-grade. GAP-002 (multipart) is high visibility. GAP-004 (webhook signing) is a security gap that should be addressed before launch.
