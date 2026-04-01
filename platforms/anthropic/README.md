# Anthropic Integration Guide

Connects to the Anthropic API for state-of-the-art LLM capabilities using the Claude model family.

## Setup & Authentication
1. Generate an API Key in the [Anthropic Console](https://console.anthropic.com/).
2. In FerroFlux, create a new Connection and add the following to the `headers` section:
    - `x-api-key`: `YOUR_API_KEY`
    - `anthropic-version`: `2023-06-01` (or latest)

## Available Actions

### `messages.create`
Sends a message to the Claude model and receives a complete response.
- **Key Inputs**: 
    - `model`: (e.g., `claude-3-5-sonnet-20240620`).
    - `messages`: An array of role/content objects.
    - `max_tokens`: Maximum response tokens.
    - `system`: (Optional) System instructions.
- **Outputs**: 
    - `response`: The full message object from Anthropic.
    - `text`: The extracted text content.

### `messages.stream`
Starts a streaming connection for real-time token delivery (useful for chat UIs).

## Examples (WAML)

### Simple Chat Completion
```waml
- step: ask_claude
  call: anthropic.messages.create
  with:
    model: "claude-3-5-sonnet-20240620"
    max_tokens: 1024
    messages:
      - role: "user"
        content: "What is the capital of France?"
```
