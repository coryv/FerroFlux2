# Twilio Integration Guide

Connects to the Twilio REST API for sending SMS, MMS, and WhatsApp messages.

## Setup & Authentication
1. Generate an Account SID and Auth Token in the [Twilio Console](https://www.twilio.com/console).
2. Buy or identify a Twilio phone number with SMS capabilities.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(ACCOUNT_SID:AUTH_TOKEN)`).

## Available Actions

### `messages.create`
Sends an SMS or MMS message to a phone number.
- **Key Inputs**: 
    - `To`: The recipient phone number (e.g., `+1234567890`).
    - `From`: Your Twilio phone number.
    - `Body`: The message text.
    - `MediaUrl`: (Optional) URL of an image/file for MMS.
- **Outputs**: 
    - `message`: The created message object.

## Examples (WAML)

### Sending an SMS Alert
```waml
- step: send_sms
  call: twilio.messages.create
  with:
    To: "+1234567890"
    From: "+1987654321"
    Body: "Warning: High server load! - " + inputs.load
```

### Sending a WhatsApp Message
*Note: Requires WhatsApp sandbox/number setup in Twilio.*
```waml
- step: send_wa
  call: twilio.messages.create
  with:
    To: "whatsapp:+1234567890"
    From: "whatsapp:+1987654321"
    Body: "Hello from FerroFlux!"
```
```
