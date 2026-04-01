# Google Gmail Integration Guide

Connects to the Gmail API for sending, searching, and managing emails in your account.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required Gmail permissions: `https://www.googleapis.com/auth/gmail.send`, `https://www.googleapis.com/auth/gmail.readonly`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `email.send`
Sends a new email message.
- **Key Inputs**: 
    - `to`: The recipient email address.
    - `subject`: The email subject.
    - `body`: The email body (HTML or plain text).
- **Outputs**: 
    - `message`: The created message object.

### `emails.search`
Searches for emails using a query string.
- **Key Inputs**: `q` (e.g., `from:jane@example.com`).

### `emails.get_attachment`
Retrieves an attachment from a specific message.

### `emails.add_label` / `emails.remove_label`
Manages labels on an email message.

## Available Triggers

### `emails.new_matching_search`
Fires when a new email matches a search query.
- **Settings**: `query`, `poll_interval`.

## Examples (WAML)

### Sending an Email
```waml
- step: send_report
  call: gmail.email.send
  with:
    to: "client@example.com"
    subject: "Your Daily Report"
    body: "Hi, please find the report attached."
```

### Searching for Invoices
```waml
- step: find_invoices
  call: gmail.emails.search
  with:
    q: "subject:Invoice filename:pdf"
```
```
