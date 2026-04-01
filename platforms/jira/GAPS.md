# Jira — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.issues.new.yaml`, `trigger.issues.updated.yaml`, and `trigger.comments.new.yaml` implement cursor-based polling via the Jira Search API (`/search` with JQL `created > cursor`).
- **Original concern:** Atlassian Connect webhooks require server-side registration and aren't portable as YAML triggers.
- **Resolution approach:** JQL-based polling provides equivalent functionality for most workflow use cases.

## Remaining Gaps

- **Real-time Webhook Triggers:** For sub-minute latency, Jira Cloud webhooks could be added as `trigger.issues.created.webhook.yaml` using Atlassian's webhook subscription API. Requires Atlassian Connect app setup.
- **Sprint/Board Events:** `sprintStarted`, `boardConfigurationChanged` — no polling equivalent; would need webhooks.
