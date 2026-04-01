# Slack Integration Guide

Connects to the Slack API to send messages, manage channels, and trigger workflows based on workspace activity.

## Setup & Authentication
To use Slack, you need a Slack App with the appropriate scopes.
1. Create a Slack App at [api.slack.com/apps](https://api.slack.com/apps).
2. Add **Bot Token Scopes** (e.g., `chat:write`, `channels:read`, `groups:read`, `im:read`, `mpim:read`, `reactions:read`).
3. Install the App to your workspace to get a **Bot User OAuth Token**.
4. In FerroFlux, create a new Connection and paste your token into the `Authorization` header field (formatted as `Bearer xoxb-YOUR_TOKEN`).

## Available Actions

### `messages.send_dm`
Sends a direct message to a user.
- **Key Inputs**: 
    - `user_id`: The ID of the user (e.g., `U123456789`).
    - `text`: The message content.
- **Outputs**: 
    - `Success`, `Error`.

### `reactions.add`
Adds an emoji reaction to a specific message.
- **Key Inputs**: 
    - `channel`: Channel ID.
    - `timestamp`: The timestamp of the message to react to.
    - `name`: Emoji name (without colons, e.g., `thumbsup`).

### `channels.list`
Lists public channels in the workspace.
- **Outputs**: 
    - `channels`: An array of channel objects.

### `files.upload`
Uploads a file to a channel.
- **Key Inputs**: 
    - `channels`: Comma-separated list of channel IDs.
    - `content`, `filename`, `filetype`.

## Available Triggers

### `messages.new`
Fires when a new message is posted in a specific channel.
- **Settings**: 
    - `channel`: The ID of the channel to poll.
    - `poll_interval`: Frequency of polling in minutes.

### `messages.mention`
Fires when the App is mentioned in a specific channel.
- **Settings**: 
    - `channel`, `bot_user_id`, `poll_interval`.

### `reactions.new`
Fires when a new reaction is added by a user.
- **Settings**: 
    - `user_id`: (Optional) Filter by a specific user.

## Examples (WAML)

### Sending a Welcome DM
```waml
- step: welcome_user
  call: slack.messages.send_dm
  with:
    user_id: inputs.new_user_id
    text: "Welcome to the team! :wave:"
```

### Reacting to a Specific Message
```waml
- step: add_check_mark
  call: slack.reactions.add
  with:
    channel: C0123456789
    timestamp: "1612345678.000100"
    name: "white_check_mark"
```
