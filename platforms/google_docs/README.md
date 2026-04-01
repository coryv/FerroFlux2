# Google Docs Integration Guide

Connects to the Google Docs API to create and manage documents in your account.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required Google Docs permissions: `https://www.googleapis.com/auth/documents.create`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `documents.create`
Creates a new Google Doc with a specified title.
- **Key Inputs**: 
    - `title`: The name of the document.
- **Outputs**: 
    - `document`: The created document object.

## Examples (WAML)

### Creating a New Document
```waml
- step: start_doc
  call: google_docs.documents.create
  with:
    title: "New Report - " + inputs.timestamp
```
```
