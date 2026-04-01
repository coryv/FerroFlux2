# Mistral Integration Guide

Connects to the Mistral AI API for open-weights and proprietary LLM models.

## Setup & Authentication
1. Generate an API Key in the [Mistral Console](https://console.mistral.ai/).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `chat.completion`
Generates a chat completion using Mistral-Small, Mistral-Medium, and Mistral-Large models.
- **Key Inputs**: 
    - `model`: (e.g., `mistral-small-latest`, `mistral-medium-latest`).
    - `messages`: An array of role/content objects.
    - `temperature`, `max_tokens`.
- **Outputs**: 
    - `response`: The full message object from Mistral.
    - `text`: The assistant's message text.

### `embeddings.create`
Generates text embeddings for vector search using Mistral-Embed.

## Examples (WAML)

### Simple Chat Completion
```waml
- step: ask_mistral
  call: mistral.chat.completion
  with:
    model: "mistral-small-latest"
    messages:
      - role: "user"
        content: "What is 2+2?"
```

### Embeddings Search
```waml
- step: embed_query
  call: mistral.embeddings.create
  with:
    input: "FerroFlux documentation"
```
```
