# Google Analytics Integration Guide

Connects to the Google Analytics Data API (GA4) to retrieve reporting and user data.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required Google Analytics permissions: `https://www.googleapis.com/auth/analytics.readonly`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
4. Set the `config.base_url` to `https://analyticsdata.googleapis.com/v1beta`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
