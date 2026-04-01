# WhatsApp Business Integration Guide

Connects to the Meta WhatsApp Business Platform API to send messages and manage customer conversations.

## Setup & Authentication
1. Generate a permanent Access Token in the [Meta App Dashboard](https://developers.facebook.com/).
2. Identify your Phone Number ID and WhatsApp Business Account ID.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
4. Set the `config.base_url` to `https://graph.facebook.com/v18.0/<phone_record_id>`.

## Available Actions

### `messages.send_text`
Sends a free-form text message to a user (within the 24-hour window).
- **Key Inputs**: 
    - `messaging_product`: `whatsapp`.
    - `to`: The recipient phone number (e.g., `1234567890`).
    - `text`: JSON object with a `body` field.
- **Outputs**: 
    - `result`: The API response.

### `messages.send_template`
Sends a pre-approved template message (required for initiating conversations).
- **Key Inputs**: 
    - `messaging_product`: `whatsapp`.
    - `template`: JSON object with `name`, `language`, and `components`.

### `messages.send_image`
Sends an image or media file to a user.

## Available Triggers

### `messages.new`
Fires when a new WhatsApp message is received via Webhook.
*Note: Requires Webhook setup with Meta.*

## Examples (WAML)

### Sending a Text Message
```waml
- step: say_hello
  call: whatsapp.messages.send_text
  with:
    messaging_product: "whatsapp"
    to: "1234567890"
    text:
      body: "Hi, how can we help today?"
```

### Sending a Template
```waml
- step: send_order_conf
  call: whatsapp.messages.send_template
  with:
    to: "1234567890"
    template:
      name: "order_confirmation"
      language:
        code: "en_US"
      components:
        - type: "body"
          parameters:
            - type: "text"
              text: inputs.order_id
```
```
