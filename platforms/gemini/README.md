# Gemini Integration Guide

Connects to the Google Gemini AI API for generative text, vision, and large-context model capabilities.

## Setup & Authentication
1. Generate an API Key in the [Google AI Studio](https://aistudio.google.com/).
2. In FerroFlux, create a new Connection and add the following:
    - `x-goog-api-key`: `YOUR_API_KEY`
    - `Content-Type`: `application/json`

## Available Actions

### `ai.generate_content`
Generates a response based on the provided prompt and context.
- **Key Inputs**: 
    - `model`: (e.g., `gemini-1.5-pro-002`, `gemini-1.5-flash-002`).
    - `contents`: An array of parts (role/text).
    - `generationConfig`: (Optional) `temperature`, `maxOutputTokens`.
- **Outputs**: 
    - `candidates`: An array of model response candidates.
    - `text`: The assistant's text content.

### `ai.generate_with_vision`
Generates a response from text and image/video inputs.
- **Key Inputs**: 
    - `contents`: Includes `inline_data` for base64-encoded images.

### `ai.embed_text`
Generates text embeddings for vector search.

### `ai.count_tokens`
Returns the token count for a given content string.

## Examples (WAML)

### Simple Text Generation
```waml
- step: ask_gemini
  call: gemini.ai.generate_content
  with:
    model: "gemini-1.5-flash-002"
    contents:
      - role: "user"
        parts:
          - text: "What is the capital of Japan?"
```

### Vision Completion
```waml
- step: analyze_image
  call: gemini.ai.generate_with_vision
  with:
    model: "gemini-1.5-flash-002"
    contents:
      - role: "user"
        parts:
          - text: "Describe this image."
          - inline_data:
              mime_type: "image/png"
              data: inputs.base64_image
```
