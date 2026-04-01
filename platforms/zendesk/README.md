# Zendesk Support Integration Guide

Connects to the Zendesk Support API to manage tickets, organizations, and users.

## Setup & Authentication
1. Generate an API Token in your [Zendesk Admin Center](https://support.zendesk.com/hc/en-us/articles/4408846332954-Generating-Zendesk-API-tokens).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(EMAIL/token:API_TOKEN)`).

## Available Actions

### `tickets.create`
Creates a new support ticket in Zendesk.
- **Key Inputs**: 
    - `subject`: Ticket subject.
    - `comment`: (Optional) Initial comment text.
    - `priority`: (Optional) `low`, `normal`, `high`, `urgent`.
    - `requester`: (Optional) `email`, `name`.
- **Outputs**: 
    - `ticket`: The created ticket object.

### `tickets.update`
Updates an existing ticket's status, assignee, or comments.
- **Key Inputs**: 
    - `id`: The ID of the ticket.
    - `status`: (Optional) `open`, `pending`, `solved`, `closed`.

### `organizations.list`
Lists all organizations in your Zendesk account.

### `users.get`
Retrieves a user's details by their ID or email.

## Available Triggers

### `tickets.new`
Fires when a new ticket is created.
- **Settings**: `poll_interval`.

## Examples (WAML)

### Creating a Support Ticket
```waml
- step: create_bug_ticket
  call: zendesk.tickets.create
  with:
    subject: "Bug: App crash on startup"
    comment: "Reported by " + inputs.user_email
    priority: "high"
```

### Closing a Ticket
```waml
- step: solve_ticket
  call: zendesk.tickets.update
  with:
    id: "12345"
    status: "solved"
```
```
