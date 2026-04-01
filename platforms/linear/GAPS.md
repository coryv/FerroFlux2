# Linear — Integration Gaps

## Triggers

- **Status:** Resolved (polling + webhook) — `trigger.issues.new.yaml` and `trigger.issues.updated.yaml` poll the Linear GraphQL API with cursor-based filtering. Webhook triggers (`trigger.issues.created.webhook.yaml`, `trigger.issues.updated.webhook.yaml`) are also available using `Linear-Signature` HMAC-SHA256 verification.

## Remaining Gaps

- **Comment Triggers:** `trigger.comments.new.yaml` not yet implemented.
- **Project/Cycle Events:** No triggers for project or cycle state changes yet.
