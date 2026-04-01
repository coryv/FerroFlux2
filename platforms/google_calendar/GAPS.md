# Google Calendar — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.events.new.yaml` and `trigger.events.updated.yaml` poll `GET /calendar/v3/calendars/{id}/events` using the `updatedMin` parameter with cursor state.
- **Original concern:** Google Push Notifications require server-side channel registration and domain verification. The `events.list` REST API supports polling with `updatedMin`.

## Remaining Gaps

- **Google Push Notifications:** Real-time triggers via `channels.watch` require server-side channel lifecycle management (registration, renewal, stop). Future engine work.
- **Event Deletion Trigger:** The polling approach captures deleted events only if `showDeleted=true` is set — `status: "cancelled"` items are included in results.
