# WhatsApp Business — Integration Gaps

## New Message Trigger (`webhook`)

- **Status:** Resolved — `trigger.messages.new.yaml` implements inbound webhook trigger using Meta's webhook payload structure.
- **API endpoint:** `POST /webhooks/whatsapp`
- **Docs:** https://developers.facebook.com/docs/whatsapp/cloud-api/guides/set-up-webhooks
- **Resolved by:** GAP-004 (Webhook Signature Verification) is implemented. The trigger extracts the message from `entry[0].changes[0].value.messages[0]` and fires per message received.

## Remaining Gaps

- **Webhook Verification Handshake:** Meta sends a GET request with `hub.challenge` during webhook registration. This is a server-level concern requiring the FerroFlux HTTP layer to echo the challenge. Tracked in SYSTEM_GAPS.md (GAP-007 candidate).
- **Status Messages / Delivery Receipts:** Not yet implemented — could be added as `trigger.messages.status.yaml`.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
