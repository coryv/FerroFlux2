# Dropbox Integration Guide

Connects to the Dropbox API for managing files and folders within your account.

## Setup & Authentication
1. Generate an API Key (Access Token) in the [Dropbox App Console](https://www.dropbox.com/developers/apps).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
3. Set the `Dropbox-API-Arg` and `Content-Type` headers for specific actions (e.g., `application/octet-stream` for uploads).

## Available Actions

### `files.upload`
Uploads a file or content to a Dropbox folder.
- **Key Inputs**: 
    - `path`: The file path in Dropbox (e.g., `/Apps/FerroFlux/Report.pdf`).
    - `content`: Base64-encoded file content.
- **Outputs**: 
    - `file`: The uploaded file metadata.

### `files.download`
Retrieves a file's content from Dropbox.
- **Key Inputs**: 
    - `path`: The file path in Dropbox.
- **Outputs**: 
    - `content`: Base64-encoded file content.

### `files.list`
Lists files and folders in a specific Dropbox path.
- **Key Inputs**: 
    - `path`: The folder path.
- **Outputs**: 
    - `files`: An array of file and folder metadata.

### `files.delete`
Permanently deletes a file or folder from Dropbox.

## Examples (WAML)

### Uploading a Text File
```waml
- step: save_log
  call: dropbox.files.upload
  with:
    path: "/logs/daily_log.txt"
    content: "SGVsbG8gV29ybGQh" # "Hello World!"
```

### Listing Files in a Shared Folder
```waml
- step: list_shared_docs
  call: dropbox.files.list
  with:
    path: "/shared_docs"
```
```
