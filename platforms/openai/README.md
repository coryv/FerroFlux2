# OpenAI Integration Guide

Connects to the OpenAI API for LLM completions, vision, and tool-calling models.

## Setup & Authentication
1. Generate an API Key in the [OpenAI Platform](https://platform.openai.com/).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `chat.completions`
Generates a chat completion using GPT models.
- **Key Inputs**: 
    - `model`: (e.g., `gpt-4o`, `gpt-4-turbo`).
    - `messages`: An array of chat message objects (role/content).
    - `temperature`, `max_tokens`.
- **Outputs**: 
    - `response`: The full response object from OpenAI.
    - `text`: The assistant's message content.

### `chat.completions_with_tools`
Generates a completion while allowing the model to call FerroFlux actions as tools.
- **Key Inputs**: 
    - `tools`: An array of tool definitions.
    - `tool_choice`: (Optional) `auto`, `none`.

## Examples (WAML)

### Simple Chat Completion
```waml
- step: ask_gpt
  call: openai.chat.completions
  with:
    model: "gpt-4o"
    messages:
      - role: "user"
        content: "What is 2+2?"
```

### Vision Completion
Use `gpt-4o` with base64-encoded images in the message content to perform vision tasks.
