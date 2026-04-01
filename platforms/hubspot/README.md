# HubSpot Integration Guide

Connects to the HubSpot CRM API to manage deals, contacts, and customer data.

## Setup & Authentication
1. Generate an Access Token in your [HubSpot Developer Settings](https://app.hubspot.com/settings/apps).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `deals.create`
Creates a new deal in a specific pipeline.
- **Key Inputs**: 
    - `dealname`: Deal title.
    - `amount`: (Number) Deal value.
    - `pipeline`: Pipeline ID.
    - `dealstage`: Stage ID (e.g., `appointmentscheduled`).
- **Outputs**: 
    - `deal`: The created deal object.

### `contacts.create`
Creates a new contact in HubSpot.
- **Key Inputs**: `email`, `firstname`, `lastname`, `phone`.

### `contacts.search`
Search for contacts using filters.
- **Key Inputs**: `email` (Optional).

## Examples (WAML)

### Creating a Deal
```waml
- step: new_deal
  call: hubspot.deals.create
  with:
    dealname: "New Project - " + inputs.customer_name
    amount: inputs.budget
    pipeline: "default"
    dealstage: "appointmentscheduled"
```

### Searching for a Contact
```waml
- step: find_contact
  call: hubspot.contacts.search
  with:
    email: "user@example.com"
```
```
