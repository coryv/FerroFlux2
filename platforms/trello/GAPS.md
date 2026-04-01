# Trello — Integration Gaps

## Triggers

- **Status:** Resolved (polling) — `trigger.cards.new.yaml` and `trigger.cards.moved.yaml` poll `GET /1/boards/{boardId}/actions` with a `since` cursor for `createCard` and `updateCard:idList` action types.
- **Original concern:** Trello's outbound webhook model requires server-side registration. The REST actions API provides an equivalent polling approach.

## Remaining Gaps

- **Real-time Webhook Triggers:** Trello supports registering webhooks via `POST /1/webhooks`. A `trigger.cards.created.webhook.yaml` could be added for sub-minute latency.
- **Board/List Events:** No triggers for list creation or board-level events yet.
