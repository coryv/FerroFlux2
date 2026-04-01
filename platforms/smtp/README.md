# SMTP Email Integration Guide

Connects to a generic SMTP server for sending email notifications.

## Setup & Authentication
1. In FerroFlux, create a new Connection with the following configuration:
    - `host`: SMTP server address (e.g., `smtp.gmail.com`).
    - `port`: Default is `587` (TLS) or `465` (SSL).
    - `username`: The sender email address.
    - `password`: The SMTP password or App Password.
    - `use_tls`: (Boolean) `true` for TLS/STARTTLS.

## Available Actions

### `email.send`
Sends a new email message via SMTP.
- **Key Inputs**: 
    - `to`: The recipient email address.
    - `from`: (Optional) The sender email address.
    - `subject`: The email subject.
    - `body`: The email body (plain text or HTML).
- **Outputs**: 
    - `success`: (Boolean) Indicates if the email was accepted by the server.

## Examples (WAML)

### Sending an Alert
```waml
- step: send_alert
  call: smtp.email.send
  with:
    to: "admin@example.com"
    subject: "Server Overload!"
    body: "The server at " + inputs.host + " is currently overloaded."
```

### Sending an HTML Report
```waml
- step: email_report
  call: smtp.email.send
  with:
    to: "reports@example.com"
    subject: "Monthly Report"
    body: "<h1>Report Card</h1><p>V1.0 Summary...</p>"
```
```
