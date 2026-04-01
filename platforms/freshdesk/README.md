# Freshdesk Integration Guide

Connects to the Freshdesk Support API for customer service and ticketing.

## Setup & Authentication
1. Generate an API Key in your [Freshdesk Profile Settings](https://support.freshdesk.com/en/support/solutions/articles/215517).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(API_KEY:X)`).
3. Set the `config.base_url` to `https://<domain>.freshdesk.com/api/v2/`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
