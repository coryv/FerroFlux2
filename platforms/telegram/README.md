# Telegram Bot Integration Guide

Connects to the Telegram Bot API to send messages, media, and polls to users or groups.

## Setup & Authentication
1. Generate a Bot Token from [@BotFather](https://t.me/botfather).
2. Start a chat with your bot or add it to a group.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bot YOUR_BOT_TOKEN`).
4. Set the `config.base_url` to `https://api.telegram.org/bot<token>`.

## Available Actions

### `messages.send`
Sends a text message to a user or chat.
- **Key Inputs**: 
    - `chat_id`: The ID of the chat.
    - `text`: The message text.
    - `parse_mode`: (Optional) `MarkdownV2` or `HTML`.
- **Outputs**: 
    - `message`: The created message object.

### `media.send_photo`
Sends a photo or image to a chat.
- **Key Inputs**: `chat_id`, `photo` (URL or File ID), `caption`.

### `interaction.send_poll`
Sends a native poll to a chat.
- **Key Inputs**: `chat_id`, `question`, `options` (Array of strings).

## Available Triggers

### `messages.new`
Fires when a new message is received by the bot.
- **Settings**: `poll_interval`.

### `messages.command`
Fires when a specific command (e.g., `/start`) is used.

## Examples (WAML)

### Sending a Text Alert
```waml
- step: notify_group
  call: telegram.messages.send
  with:
    chat_id: "-100123456789"
    text: "Alert: New lead created! - " + inputs.lead_name
```

### Sending a Poll
```waml
- step: lunch_vote
  call: telegram.interaction.send_poll
  with:
    chat_id: "-100123456789"
    question: "Where should we go for lunch?"
    options: ["Pizza", "Burgers", "Salad"]
```
```
