# Notion — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.database.new_row.yaml` polls a Notion database for new rows using `POST /databases/{id}/query` filtered by `created_time` with cursor state.
- **Original concern:** Notion does not support webhooks natively. Polling resolves the core use case for new-row workflows.

## Remaining Gaps

- **Updated Row Trigger:** Notion does not allow efficient server-side filtering by `last_edited_time` via the query API — all rows must be fetched to detect edits.
- **Real-time:** Notion has no webhook or push notification system. Polling with short intervals is the only approach.
