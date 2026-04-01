# Microsoft Outlook Integration Guide

Connects to the Microsoft Graph API for managing emails, calendars, and contacts in your Outlook or Office 365 account.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Azure App Registrations](https://portal.azure.com/).
2. Add the required Microsoft Graph permissions: `Mail.Send`, `Calendars.ReadWrite`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `messages.send`
Sends a new email message.
- **Key Inputs**: 
    - `to_recipients`: Array of email recipient objects.
    - `subject`: The email subject.
    - `body`: The email body (HTML or plain text).
- **Outputs**: 
    - `result`: The API response.

### `calendar.create`
Creates a new event in an Outlook calendar.
- **Key Inputs**: 
    - `subject`, `start_time`, `end_time`, `location`.

### `messages.list`
Lists emails in your Inbox.

### `messages.reply`
Sends a reply to an existing email message.

## Available Triggers

### `messages.new`
Fires when a new email is received.
- **Settings**: `poll_interval`.

### `calendar.new`
Fires when a new calendar event is created.

## Examples (WAML)

### Sending an Email
```waml
- step: send_invite
  call: outlook.messages.send
  with:
    to_recipients: [{emailAddress: {address: "client@example.com"}}]
    subject: "Interview Invitation"
    body: "Hi, please pick a slot in the calendar."
```

### Scheduling a Meeting
```waml
- step: schedule_sync
  call: outlook.calendar.create
  with:
    subject: "Sync - " + inputs.team_name
    start_time: inputs.start
    end_time: inputs.end
```
```
