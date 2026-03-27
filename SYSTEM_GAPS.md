# FerroFlux System Gaps

This file tracks execution engine capabilities that are missing or insufficient to fully implement high-value integration nodes. Each entry includes the affected integrations, what specifically is blocked, and what system change would unblock it.

Per-integration omissions (individual nodes that couldn't be built) are documented in `platforms/<platform_id>/GAPS.md`. This file is for **cross-cutting system-level gaps** that affect multiple integrations simultaneously.

---

## GAP-001 — OAuth2 Token Refresh

**Priority:** Critical
**Status:** Implemented

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

### Implementation notes
- `crates/FerroFlux-core/src/oauth2.rs` — `TokenRefreshLocks` (Bevy ECS Resource), `resolve_oauth2_token()` sync entry point, `refresh_access_token()` async POST, `is_token_expired()` with 60-second buffer
- `crates/FerroFlux-core/src/secrets.rs` — `update_connection_data()` re-encrypts and persists refreshed tokens
- `crates/FerroFlux-core/src/store/database.rs` — `update_connection_encrypted_data()` targeted SQL UPDATE
- `crates/FerroFlux-core/src/tools/primitives/request.rs` — `"OAuth2"` auth arm in `resolve_connection_auth()` calls `resolve_oauth2_token()`
- Transparent to YAML authors — works automatically for any connection with `auth_type: OAuth2`

---

## GAP-002 — Multipart/Form-Data Request Bodies

**Priority:** High
**Status:** Implemented

### What's missing
`http_client` only supports JSON request bodies (`Content-Type: application/json`). It cannot construct `multipart/form-data` requests required for file and binary uploads.

### What breaks without it
All file upload nodes must be omitted from integrations. Users cannot upload files to any platform through FerroFlux workflows.

### Affected integrations (partial list)
Slack (Upload File), Google Drive (Upload File), Dropbox (Upload File), AWS S3 (Upload Object), GitHub (Upload Release Asset), Gemini File API (Upload File), AssemblyAI (Upload Audio), ElevenLabs (Upload Voice), Cloudinary (Upload Image/Video), OneDrive (Upload File).

### What's needed
- `http_client` support for `body_type: multipart` with named parts, file references, and per-part `Content-Type` headers
- Ability to reference a binary value from a previous node's output as a part body

### Implementation notes
- `crates/FerroFlux-core/Cargo.toml` — added `multipart` to reqwest features
- `crates/FerroFlux-core/src/tools/primitives/request.rs` — `build_multipart_form(parts, context)` resolves each part from `content` (literal), `content_json` (serialized), or `content_var` (variable name in `context.local`, supports `DataRef::Blob` via BlobStore); `execute_multipart_request()` sends the form
- `crates/FerroFlux-core/src/tools/primitives/http_client.rs` — detects `body_type: "multipart"`, branches to new helpers; existing JSON path unchanged

---

## GAP-003 — Pagination / Cursor Iteration

**Priority:** High
**Status:** Implemented

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

### Implementation notes
- `crates/FerroFlux-core/src/tools/primitives/paginate.rs` — `PaginateTool` (tool ID: `"paginate"`) with all four strategies; returns `{ items, total_pages, total_items }`
- `crates/FerroFlux-core/src/tools/primitives/request.rs` — shared helpers (`extract_by_path`, `set_query_param`, `parse_link_header_next`) used by `PaginateTool`
- Registered in `register_core_tools()` — drop-in replacement for `http_client` on any list endpoint; no YAML schema changes required

---

## GAP-004 — Webhook Signature Verification

**Priority:** High
**Status:** Implemented

### What's missing
The core `trigger.webhook` node accepts any inbound HTTP request without verifying its authenticity. There is no built-in step for HMAC-SHA256 (or similar) signature verification against a shared secret.

### What breaks without it
Webhook-triggered workflows are insecure — any party that knows the webhook URL can trigger them. Platform-specific verification (Stripe's `Stripe-Signature`, GitHub's `X-Hub-Signature-256`, Slack's `X-Slack-Signature`, Shopify's `X-Shopify-Hmac-Sha256`) must be either skipped or re-implemented ad hoc in each integration.

### Affected integrations (partial list)
Stripe, GitHub, Slack, Shopify, HubSpot, Linear, Twilio, Intercom, WooCommerce, PagerDuty, and any platform that signs outbound webhook payloads.

### What's needed
- A `verify_signature` execution tool (or a setting on `trigger.webhook`) that takes a raw body, a secret, an algorithm (`hmac-sha256`, `hmac-sha1`), and an expected signature header — and routes to Error if verification fails
- Optionally, platform-level declarations of the signature scheme so trigger nodes can self-configure

### Implementation notes
- `crates/FerroFlux-core/Cargo.toml` — added `hmac`, `sha2`, `sha1` (RustCrypto)
- `crates/FerroFlux-core/src/tools/primitives/verify_signature.rs` — `VerifySignatureTool` (tool ID: `"verify_signature"`); params: `body`, `secret`, `signature`, `algorithm` (`hmac-sha256`/`hmac-sha1`), `encoding` (`hex`/`base64`); auto-strips platform prefixes (`sha256=`, `v0=`, etc.); uses constant-time `verify_slice` comparison
- `platforms/core/trigger.webhook.yaml` — added `raw_body` output port and emit step; the HTTP layer is expected to include `event.raw_body` in the trigger payload

---

## GAP-005 — Streaming / Server-Sent Events (SSE)

**Priority:** Medium-High
**Status:** Partially Implemented

### What's missing
`http_client` waits for the full response body before returning. It cannot handle streaming responses (`Transfer-Encoding: chunked`, `Content-Type: text/event-stream`) where tokens or events arrive incrementally.

### What breaks without it
AI completion nodes block until the full response is received, which for long outputs can be many seconds with no feedback. Streaming is the expected UX for LLM integrations. Real-time event feeds (SSE-based triggers) also cannot be implemented.

### Affected integrations (partial list)
OpenAI (Streaming Chat Completion), Anthropic Claude (Streaming Messages), Google Gemini (Streaming Generate), Azure OpenAI (Streaming), Mistral, Groq, Cohere — and the SSE trigger node in the roadmap.

### What's needed
- `http_client` support for `stream: true` that emits partial results incrementally as a node output stream
- A `trigger.sse` node type that connects to an SSE endpoint and fires the workflow on each event

### Implementation notes (partial)

**`http_client` `stream: true` — Implemented:**
- `crates/FerroFlux-core/src/api/events.rs` — added `SystemEvent::StreamChunk { trace_id, step_id, chunk, done }` variant; SDK clients subscribe to receive tokens as they arrive
- `crates/FerroFlux-core/src/tools/primitives/request.rs` — `execute_streaming_request()` wraps `reqwest::blocking::Response` (which implements `std::io::Read`) in a `BufReader`; reads line-by-line; parses SSE `data:` fields; extracts text from OpenAI (`choices[0].delta.content`), Anthropic (`delta.text`), and Gemini (`candidates[0].content.parts[0].text`) formats; emits `StreamChunk` events in real-time; accumulates full text; returns `{ chunks, text, total_chunks }`
- `crates/FerroFlux-core/src/tools/primitives/http_client.rs` — detects `stream: true` param and dispatches to `execute_streaming_request()`; optional `step_id` param for chunk event correlation
- YAML usage: `stream: true`, optional `step_id: <name>` for event correlation

**`trigger.sse` — Schema defined, runtime not yet implemented:**
- `platforms/core/trigger.sse.yaml` — interface contract: outputs `data`, `raw`, `event_type`, `id`; settings: `url`, `connection`, `headers`, `reconnect_delay_ms`, `max_reconnect_attempts`
- **Runtime gap**: Requires a persistent outbound connection manager — a Bevy system that spawns and supervises SSE reader tasks (one per registered trigger), dispatching workflow executions per event. This differs fundamentally from webhook triggers (passive/inbound) and requires dedicated runtime work beyond the tool/pipeline layer.

---

## GAP-006 — Binary / Raw Request Bodies

**Priority:** Medium
**Status:** Implemented

### What's missing
`http_client` cannot send raw binary bodies (e.g., `Content-Type: audio/mpeg`, `image/png`, `application/octet-stream`). Only JSON bodies are supported.

### What breaks without it
APIs that accept raw binary input — rather than base64-encoded JSON or multipart — cannot be called directly.

### Affected integrations (partial list)
ElevenLabs (Send Audio for Speech-to-Speech), AssemblyAI (Upload raw audio), Stability AI (Send image for img2img), Replicate (binary media inputs), Deepgram (audio upload).

### What's needed
- `http_client` support for `body_type: binary` with a `content_type` field and a binary value reference from a prior step output

### Implementation notes
- `crates/FerroFlux-core/src/tools/primitives/request.rs` — `execute_binary_request()` sets `Content-Type` header and sends raw bytes via `req.body(bytes)`
- `crates/FerroFlux-core/src/tools/primitives/http_client.rs` — `body_type: "binary"` branch calls `resolve_binary_body()` (private fn); resolves from `body_var` (context local: `DataRef::Blob` via BlobStore claim, `DataRef::Inline(String)` as UTF-8 bytes, `DataRef::Inline(other)` serialized as JSON bytes) or `body_base64` (decoded); errors if neither present
- YAML usage: `body_type: binary`, `body_var: <var_name>` or `body_base64: <encoded>`, `content_type: <mime>`

---

## Summary

| ID | Gap | Priority | Status | Integrations affected |
|---|---|---|---|---|
| GAP-001 | OAuth2 token refresh | Critical | ✅ Implemented | ~30+ |
| GAP-002 | Multipart/form-data bodies | High | ✅ Implemented | ~10+ |
| GAP-003 | Pagination / cursor iteration | High | ✅ Implemented | ~20+ |
| GAP-004 | Webhook signature verification | High | ✅ Implemented | ~10+ |
| GAP-005 | Streaming / SSE | Medium-High | ⚠️ Partial (http_client done; trigger.sse needs runtime) | ~6+ AI + real-time feeds |
| GAP-006 | Binary raw bodies | Medium | ✅ Implemented | ~5+ |

**Recommended build order:** GAP-001 → GAP-003 → GAP-002 → GAP-004 → GAP-006 → GAP-005

GAP-001 (OAuth2) and GAP-003 (pagination) together make the most existing and planned integrations production-grade. GAP-002 (multipart) is high visibility. GAP-004 (webhook signing) is a security gap that should be addressed before launch.
