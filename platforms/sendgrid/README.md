# SendGrid Integration Guide

Connects to the Twilio SendGrid v3 API for high-volume email delivery and transactional messaging.

## Setup & Authentication
1. Generate an API Key in your [SendGrid Settings](https://app.sendgrid.com/settings/api_keys).
2. Add the required SendGrid permissions: `Mail Send`, `Read Account`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).
4. Set the `config.base_url` to `https://api.sendgrid.com/v3`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
