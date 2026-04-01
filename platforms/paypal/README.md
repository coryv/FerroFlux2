# PayPal Integration Guide

Connects to the PayPal REST API for payments, subscriptions, and payouts.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [PayPal Developer Portal](https://developer.paypal.com/dashboard/applications).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(CLIENT_ID:CLIENT_SECRET)`).
3. Set the `config.base_url` to `https://api-m.paypal.com/v1` (Live) or `https://api-m.sandbox.paypal.com/v1` (Sandbox).

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
