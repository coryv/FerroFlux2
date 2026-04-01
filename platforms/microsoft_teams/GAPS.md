# Microsoft Teams — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.messages.new.yaml` polls `GET /v1.0/teams/{teamId}/channels/{channelId}/messages/delta` using a `createdDateTime` filter with cursor state.
- **Original concern:** MS Graph Change Notifications require webhook endpoint verification via subscription API. The delta API provides polling-based access without server-side subscription management.

## Remaining Gaps

- **Direct Message Triggers:** DMs (chats) use a different Graph API endpoint (`/chats/{chatId}/messages`) — not yet implemented.
- **Change Notification Webhooks:** Real-time via MS Graph subscription API is a future addition requiring server-side subscription lifecycle management.
