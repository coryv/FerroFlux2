# Ollama Cloud Integration Guide

Connects to the Ollama Cloud API for managed, open-weights LLM inference using the Ollama model format.

## Setup & Authentication
1. Generate an API Key in the [Ollama Cloud Console](https://ollama.cloud/).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `ai.chat`
Generates a chat completion using Llama-3, Gemma, Mistral, and other Ollama-compatible models.
- **Key Inputs**: 
    - `model`: (e.g., `llama3.1`, `gemma2`).
    - `messages`: An array of role/content objects.
    - `stream`: Boolean (whether to return tokens as they are generated).
- **Outputs**: 
    - `response`: The full message object from Ollama.
    - `message`: The assistant's message content.

### `ai.list_models`
Lists all models currently available in your Ollama Cloud project.

## Examples (WAML)

### Simple Chat Completion
```waml
- step: ask_ollama
  call: ollama_cloud.ai.chat
  with:
    model: "llama3.1:70b"
    messages:
      - role: "user"
        content: "What is the capital of Italy?"
```

### List Available Models
```waml
- step: check_models
  call: ollama_cloud.ai.list_models
```
```
