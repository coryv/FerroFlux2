# Google Drive Integration Guide

Connects to the Google Drive API for managing files and folders within your account or shared drives.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Google Cloud Console](https://console.cloud.google.com/).
2. Add the required scopes: `https://www.googleapis.com/auth/drive.file`, `https://www.googleapis.com/auth/drive.readonly`.
3. In FerroFlux, create a new Connection and add the following:
    - `apiUrl`: `https://www.googleapis.com/drive/v3`
    - `Authorization`: `Bearer YOUR_ACCESS_TOKEN`

## Available Actions

### `files.upload`
Uploads a file or content to a Google Drive folder.
- **Key Inputs**: 
    - `name`: File name.
    - `content`: Base64-encoded file content.
    - `mimeType`: (Optional) MIME type.
    - `parents`: (Optional) Folder IDs.
- **Outputs**: 
    - `file`: The created file object.

### `files.download`
Retrieves a file's content from Google Drive.
- **Key Inputs**: 
    - `fileId`: The ID of the file to download.
- **Outputs**: 
    - `content`: Base64-encoded file content.

### `files.list`
Lists files and folders in your Google Drive.
- **Key Inputs**: 
    - `q`: (Optional) Query string (e.g., `name contains 'Report'`).
- **Outputs**: 
    - `files`: An array of file metadata.

### `files.delete`
Moves a file or folder to the trash.

### `folders.create`
Creates a new folder in Google Drive.

## Examples (WAML)

### Searching for a File
```waml
- step: find_report
  call: google-drive.files.search
  with:
    q: "name contains 'Invoice' and mimeType = 'application/pdf'"
```

### Uploading to a Folder
```waml
- step: upload_result
  call: google-drive.files.upload
  with:
    name: "Result.txt"
    content: "SGVsbG8gV29ybGQh" # "Hello World!"
    parents: ["root"]
```
```
