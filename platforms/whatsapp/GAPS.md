# WhatsApp Business — Integration Gaps

## New Message Trigger (`webhook`)

- **Why omitted:** FerroFlux does not currently support inbound HTTP webhooks for external platforms.
- **API endpoint:** `POST /webhooks/whatsapp`
- **Docs:** https://developers.facebook.com/docs/whatsapp/cloud-api/guides/set-up-webhooks
- **Value:** High — essential for interactive chat bots.
- **Unblocked by:** GAP-005 (Inbound Webhook Engine).

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
