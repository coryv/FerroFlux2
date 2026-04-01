# Intercom Integration Guide

Connects to the Intercom API for managing customer conversations, contacts, and segments.

## Setup & Authentication
1. Generate an Access Token in your [Intercom Developer Hub](https://developers.intercom.com/).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
3. Set the `Accept` header to `application/json` and `Intercom-Version` to your desired version.
4. Set the `config.base_url` to `https://api.intercom.io`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
