# Atlassian Confluence Integration Guide

Connects to the Confluence Cloud API to manage pages, spaces, and attachments.

## Setup & Authentication
1. Generate an API Token in your [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(EMAIL:API_TOKEN)`).

## Available Actions

### `pages.create`
Creates a new page in a specific space.
- **Key Inputs**: 
    - `spaceKey`: Space key (e.g., `DEV`).
    - `parent_id`: (Optional) Parent page ID.
    - `title`: Page title.
    - `content`: HTML or Storage format content.
- **Outputs**: 
    - `page`: The created page object.

### `pages.update`
Updates an existing page's content or title.
- **Key Inputs**: 
    - `pageId`, `title`, `content`, `version` (Number).

### `pages.add_attachment`
Uploads a file as an attachment to a page.

### `spaces.list`
Lists all spaces in the Confluence instance.

## Available Triggers

### `pages.new`
Fires when a new page is created in a space.
- **Settings**: `space_key`, `poll_interval`.

## Examples (WAML)

### Creating a Meeting Note
```waml
- step: create_note
  call: confluence.pages.create
  with:
    spaceKey: "PROJECT"
    title: "Meeting Notes - " + inputs.date
    content: "<h1>Minutes</h1><p>" + inputs.summary + "</p>"
```

### Updating a Page
```waml
- step: update_doc
  call: confluence.pages.update
  with:
    pageId: "123456"
    title: "Updated Project Specs"
    content: "<p>V2 Specs...</p>"
    version: inputs.current_version + 1
```
```
