# Microsoft Teams Integration Guide

Connects to the Microsoft Graph API to send messages and notifications to Teams channels.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Azure App Registrations](https://portal.azure.com/).
2. Add the required Microsoft Graph permissions: `ChannelMessage.Send`, `Group.ReadWrite.All`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `messages.send`
Sends a new message to a specific Teams channel.
- **Key Inputs**: 
    - `team_id`: The ID of the Team.
    - `channel_id`: The ID of the channel.
    - `content`: The message text (HTML or plain text).
- **Outputs**: 
    - `message`: The created message object.

## Examples (WAML)

### Sending a Channel Alert
```waml
- step: notify_team
  call: microsoft_teams.messages.send
  with:
    team_id: "12345-abcde"
    channel_id: "98765-fghij"
    content: "Alert: Deployment successful! - " + inputs.env
```
```
