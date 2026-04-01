# Twilio — Integration Gaps

## Triggers

- **Status:** Resolved (webhook) — `trigger.sms.received.yaml` and `trigger.calls.incoming.yaml` implement inbound webhook triggers with `X-Twilio-Signature` HMAC-SHA1 verification. The URL-encoded form body is parsed via Rhai.
- **Original concern:** Required inbound webhook routing — now implemented via GAP-004.

## Remaining Gaps

- **Twilio Signature URL Component:** Twilio's HMAC-SHA1 is computed over the full request URL + sorted POST params concatenated. The current `verify_signature` tool signs only the body. For production use, the Twilio signature algorithm may need a dedicated tool that includes the URL. This is a known limitation — document in SYSTEM_GAPS.md.
- **Status Callbacks:** Call/SMS delivery status callbacks — could be `trigger.sms.status.yaml`.
