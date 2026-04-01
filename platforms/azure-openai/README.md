# Azure OpenAI Integration Guide

Connects to the Microsoft Azure OpenAI service for enterprise-grade LLM completions, vision, and tool-calling models.

## Setup & Authentication
1. Generate an API Key and Deployment name in the [Azure OpenAI Studio](https://oai.azure.com/).
2. In FerroFlux, create a new Connection and add the following to the `headers` and `config`:
    - `api-key`: `YOUR_AZURE_API_KEY`
    - `endpoint`: `https://YOUR_RESOURCE_NAME.openai.azure.com/`
    - `api-version`: `2024-02-15-preview` (or latest)

## Available Actions

### `chat.completion`
Generates a chat completion using GPT-4o, GPT-3.5, and other deployed models.
- **Key Inputs**: 
    - `deployment_name`: The name of the model deployment in Azure.
    - `messages`: An array of chat message objects (role/content).
    - `temperature`, `max_tokens`.
- **Outputs**: 
    - `response`: The full response object from Azure.
    - `text`: The assistant's message content.

### `chat.completion_vision`
Generates a completion from text and image/video inputs.
- **Key Inputs**: 
    - `deployment_name`: (e.g., `gpt-4-vision-preview`).

### `deployments.list`
Lists all active model deployments in the Azure OpenAI resource.

## Examples (WAML)

### Simple Chat Completion
```waml
- step: ask_azure
  call: azure-openai.chat.completion
  with:
    deployment_name: "gpt-4o"
    messages:
      - role: "user"
        content: "What is 2+2?"
```

### Embeddings Generation
```waml
- step: embed_query
  call: azure-openai.embeddings.create
  with:
    deployment_name: "text-embedding-3-small"
    input: "FerroFlux documentation"
```
```
