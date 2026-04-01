# Microsoft OneDrive Integration Guide

Connects to the Microsoft Graph API for managing files and folders within your OneDrive account or SharePoint libraries.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in the [Azure App Registrations](https://portal.azure.com/).
2. Add the required Microsoft Graph permissions: `Files.ReadWrite`, `Files.Read.All`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `files.upload`
Uploads a file or content to a OneDrive folder.
- **Key Inputs**: 
    - `name`: File name.
    - `content`: Base64-encoded file content.
    - `parent_id`: (Optional) Folder IDs.
- **Outputs**: 
    - `file`: The created drive item object.

### `files.download`
Retrieves a file's content from OneDrive.
- **Key Inputs**: 
    - `item_id`: The ID of the file to download.
- **Outputs**: 
    - `content`: Base64-encoded file content.

### `files.list`
Lists files and folders in your OneDrive.
- **Key Inputs**: 
    - `parent_id`: (Optional) The ID of the folder to list (default is `root`).
- **Outputs**: 
    - `files`: An array of file metadata.

### `files.delete`
Moves a file or folder to the trash.

## Examples (WAML)

### Searching for a File
```waml
- step: find_invoice
  call: onedrive.files.search
  with:
    q: "name contains 'Invoice' and mimeType = 'application/pdf'"
```

### Uploading a Report
```waml
- step: save_report
  call: onedrive.files.upload
  with:
    name: "Result.docx"
    content: "SGVsbG8gV29ybGQh" # "Hello World!"
    parent_id: "root"
```
```
