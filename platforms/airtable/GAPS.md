# Airtable — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.records.new.yaml` and `trigger.records.updated.yaml` poll using `CREATED_TIME()` and `LAST_MODIFIED_TIME()` Airtable formula filters with cursor state.
- **Original concern:** Webhook registration requires a destination URL and payload parsing. Polling via the REST API resolves the core new/updated record use cases.

## Remaining Gaps

- **Webhook Triggers:** Airtable supports webhooks (as of 2023). A real-time trigger using `POST /bases/{baseId}/webhooks` is a future addition.
- **Deleted Record Trigger:** Polling cannot detect deletions — webhook approach needed.
