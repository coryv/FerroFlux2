# Resend Integration Guide

Connects to the Resend API for high-performance email delivery and domain management.

## Setup & Authentication
1. Generate an API Key in your [Resend API Settings](https://resend.com/api-keys).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `emails.send`
Sends a new email message.
- **Key Inputs**: 
    - `from`: The sender email address (e.g., `onboarding@resend.dev`).
    - `to`: The recipient email address or array of emails.
    - `subject`: The email subject.
    - `html` / `text`: The email body content.
- **Outputs**: 
    - `id`: The ID of the sent email.

### `emails.get`
Retrieves a single email's status by its ID.
- **Key Inputs**: `id`.

### `domains.create`
Registers a new domain for sending emails.
- **Key Inputs**: `name`.

### `domains.list`
Lists all active domains in the Resend account.

## Examples (WAML)

### Sending a Welcome Email
```waml
- step: send_welcome
  call: resend.emails.send
  with:
    from: "FerroFlux <notifications@resend.dev>"
    to: inputs.user_email
    subject: "Welcome to FerroFlux, " + inputs.first_name
    html: "<p>We're glad to have you here!</p>"
```

### Checking Domain Status
```waml
- step: check_domain
  call: resend.domains.get
  with:
    id: "dom_123456789"
```
```
