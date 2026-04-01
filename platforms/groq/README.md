# Groq Integration Guide

Connects to the Groq API for ultra-fast LLM inference using the LPU™ (Language Processing Unit) family of models.

## Setup & Authentication
1. Generate an API Key in the [Groq Console](https://console.groq.com/).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `chat.completion`
Generates a chat completion using Llama-3, Mixtral, and Gemma models.
- **Key Inputs**: 
    - `model`: (e.g., `llama-3.1-70b-versatile`, `mixtral-8x7b-32768`).
    - `messages`: An array of role/content objects.
    - `temperature`, `max_tokens`.
- **Outputs**: 
    - `response`: The full message object from Groq.
    - `text`: The asistente's message text.

### `audio.transcribe`
Transcribes audio files into text using the Whisper model family.
- **Key Inputs**: 
    - `file`: The base64-encoded audio file.
    - `model`: (e.g., `whisper-large-v3`).

## Examples (WAML)

### Ultra-Fast Chat Completion
```waml
- step: fast_response
  call: groq.chat.completion
  with:
    model: "llama-3.1-8b-instant"
    messages:
      - role: "user"
        content: "What is the speed of light?"
```

### Audio Transcription
```waml
- step: transcribe_meeting
  call: groq.audio.transcribe
  with:
    model: "whisper-large-v3"
    file: inputs.base64_audio
```
