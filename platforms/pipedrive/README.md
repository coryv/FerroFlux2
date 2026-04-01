# Pipedrive Integration Guide

Connects to the Pipedrive REST API to manage deals, persons, and leads.

## Setup & Authentication
1. Generate an API Key in your [Pipedrive Personal Settings](https://pipedrive.readme.io/docs/how-to-find-the-api-token).
2. In FerroFlux, create a new Connection and add it to the `api_token` query parameter or add it to the `Authorization` header field (if using OAuth2).
3. Set the `config.base_url` to `https://<company_domain>.pipedrive.com/api/v1`.

## Status
*Note: This integration currently provides the base connection configuration. Specific action and trigger nodes are on the roadmap.*
```
