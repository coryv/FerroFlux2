# Airtable — Integration Gaps

## Triggers
- **Why omitted:** Airtable webhooks require registering webhooks with a destination URL, receiving a payload, and manually parsing it. The current integration flow doesn't have a reliable generic webhook trigger builder for inbound Airtable payloads.
- **Value:** High.
