# Mailchimp Integration Guide

Connects to the Mailchimp Marketing API to manage audiences, campaigns, and subscribers.

## Setup & Authentication
1. Generate an API Key in your [Mailchimp Account Settings](https://admin.mailchimp.com/account/api/).
2. Identify your Data Center (e.g., `us19`) from the URL of your Mailchimp dashboard.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(any:API_KEY)`).
4. Set the `config.base_url` to `https://<dc>.api.mailchimp.com/3.0`.

## Available Actions

### `subscribers.addUpdate`
Adds a new subscriber to a list or updates an existing one.
- **Key Inputs**: 
    - `list_id`: List (Audience) ID.
    - `email_address`: Contact email.
    - `status`: `subscribed`, `unsubscribed`, `cleaned`, `pending`.
    - `merge_fields`: (Optional) JSON object of merge tags (e.g., `{"FNAME": "Jane"}`).
- **Outputs**: 
    - `member`: The subscriber object.

### `campaigns.create`
Creates a new draft campaign.
- **Key Inputs**: `type` (e.g., `regular`), `recipients` (list_id).

### `campaigns.send`
Sends a draft campaign immediately.

### `tags.add`
Adds tags to a specific subscriber.

## Available Triggers

### `subscribers.new`
Fires when a new contact is added to an audience.
- **Settings**: `list_id`, `poll_interval`.

## Examples (WAML)

### Subscribing a User
```waml
- step: add_to_newsletter
  call: mailchimp.subscribers.addUpdate
  with:
    list_id: "123456"
    email_address: inputs.user_email
    status: "subscribed"
    merge_fields:
      FNAME: inputs.first_name
```

### Sending a Campaign
```waml
- step: send_alert
  call: mailchimp.campaigns.send
  with:
    campaign_id: "789012"
```
```
