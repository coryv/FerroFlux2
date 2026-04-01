# HubSpot — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.contacts.new.yaml` polls `GET /crm/v3/objects/contacts` with a `createdate` filter. `trigger.deals.updated.yaml` polls deals using `hs_lastmodifieddate`. Both use cursor state.
- **Original concern:** HubSpot Workflow Extensions rely on incoming webhooks. The CRM REST API supports efficient polling with timestamp filters.

## Remaining Gaps

- **Webhook Triggers:** HubSpot supports webhook subscriptions via the Webhooks API. Real-time triggers for contact/deal events are a future addition.
- **Deal Stage Change Trigger:** Polling for stage changes requires comparing previous state — a webhook trigger is the cleaner approach for this event type.
