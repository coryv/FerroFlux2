# Stripe — Integration Gaps

## Triggers (Payment Succeeded, etc.)
- **Why omitted:** Stripe webhook triggers push event payloads via HMAC signed `POST` requests. We lack a generic inbound webhook node that routes these back per-platform securely. The API does include a `/v1/events` cursor-paginated polling endpoint, but it is extremely noisy and rarely optimal for workflow execution.
- **Value:** High.
