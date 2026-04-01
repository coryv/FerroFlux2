# Discord Integration Guide

Connects to the Discord API to send messages and notifications to servers or direct messages.

## Setup & Authentication
1. Generate a Bot Token in the [Discord Developer Portal](https://discord.com/developers/applications).
2. Invite your bot to your server with the required permissions: `Send Messages`, `Read Messages`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bot YOUR_BOT_TOKEN`).

## Available Actions

### `messages.send`
Sends a message to a specific channel.
- **Key Inputs**: 
    - `channel_id`: The ID of the channel.
    - `content`: The message text.
    - `embeds`: (Optional) Array of rich embed objects.
- **Outputs**: 
    - `message`: The created message object.

### `messages.send_dm`
Sends a direct message to a user.
- **Key Inputs**: 
    - `user_id`: The ID of the user.
    - `content`: The message text.

## Examples (WAML)

### Sending a Server Alert
```waml
- step: notify_channel
  call: discord.messages.send
  with:
    channel_id: "123456789"
    content: "Alert: New issue reported! - " + inputs.issue_title
```

### Sending a DM
```waml
- step: private_notify
  call: discord.messages.send_dm
  with:
    user_id: "987654321"
    content: "Your report is ready."
```
```
