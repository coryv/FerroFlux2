# PostHog Integration Guide

Connects to the PostHog API for event tracking, user identification, and product analytics.

## Setup & Authentication
1. Generate a Project API Key in the [PostHog Dashboard (Project Settings)](https://app.posthog.com/project/settings).
2. Identify your instance URL (e.g., `https://app.posthog.com` or `https://eu.posthog.com`).
3. In FerroFlux, create a new Connection and add it to the `api_key` field in the `platform.yaml` config (PostHog uses API keys as properties in the event payload, not in Bearer headers for the Capture API).

## Available Actions

### `events.capture`
Captures an event performed by a user.
- **Key Inputs**: 
    - `event`: (e.g., `signed_up`).
    - `distinct_id`: The ID of the user.
    - `properties`: (Optional) JSON object of event properties.
- **Outputs**: 
    - `result`: The API response.

### `users.identify`
Identifies a user and records their properties.
- **Key Inputs**: 
    - `distinct_id`: The ID of the user.
    - `properties`: (Optional) A JSON object of user properties (e.g., `{"email": "jane@example.com"}`).

## Examples (WAML)

### Capturing a Conversion
```waml
- step: track_conversion
  call: posthog.events.capture
  with:
    event: "Conversion Completed"
    distinct_id: inputs.user_id
    properties:
      type: inputs.offer_type
      value: inputs.revenue
```

### Identifying a New Subscriber
```waml
- step: identify_user
  call: posthog.users.identify
  with:
    distinct_id: "user_123"
    properties:
      email: "user@example.com"
      plan: "Pro"
```
```
