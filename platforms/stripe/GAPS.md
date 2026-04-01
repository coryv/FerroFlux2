# Stripe — Integration Gaps

## Triggers

- **Status:** Resolved (webhook) — `trigger.payments.succeeded.yaml`, `trigger.payments.failed.yaml`, and `trigger.subscriptions.updated.yaml` implement inbound webhook triggers with `Stripe-Signature` HMAC-SHA256 verification. The Rhai script extracts the timestamp and constructs the signed payload (`{t}.{raw_body}`) before verification.
- **Original concern:** Required a generic inbound webhook with HMAC verification — now implemented via GAP-004.

## Remaining Gaps

- **Stripe-Signature Timestamp Validation:** The current implementation verifies the HMAC but does not enforce the 300-second tolerance window on the `t=` timestamp. A Rhai step could add this check.
- **Additional Event Types:** `charge.refunded`, `invoice.payment_failed`, `checkout.session.completed` — each needs its own trigger YAML (or a generic event-type-filter approach).
