# Segment Integration Guide

Connects to the Segment HTTP Tracking API to track events and identify users in your workspace.

## Setup & Authentication
1. Generate a Write Key for your Source in the [Segment App](https://app.segment.com/sources).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(WRITE_KEY:)`).

## Available Actions

### `events.track`
Tracks an action performed by a user.
- **Key Inputs**: 
    - `userId`: The ID of the user performing the action.
    - `event`: The name of the event (e.g., `Order Completed`).
    - `properties`: (Optional) A JSON object of event properties.
- **Outputs**: 
    - `result`: The API response.

### `users.identify`
Identifies a user and records their traits.
- **Key Inputs**: 
    - `userId`: The ID of the user.
    - `traits`: (Optional) A JSON object of user traits (e.g., `{"email": "jane@example.com"}`).

## Examples (WAML)

### Tracking a Conversion
```waml
- step: track_purchase
  call: segment.events.track
  with:
    userId: inputs.user_id
    event: "Purchase Completed"
    properties:
      amount: inputs.total
      currency: "USD"
```

### Identifying a New User
```waml
- step: identify_user
  call: segment.users.identify
  with:
    userId: "user_123"
    traits:
      email: "user@example.com"
      plan: "Premium"
```
```
