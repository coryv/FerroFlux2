# Notion Integration Guide

Connects to the Notion API to manage pages, databases, and blocks in your workspace.

## Setup & Authentication
1. Generate an Integration Token in your [Notion Settings](https://www.notion.so/my-integrations).
2. Share the specific pages or databases with your integration.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_INTEGRATION_TOKEN`).
4. Add the `Notion-Version` header (e.g., `2022-06-28`).

## Available Actions

### `pages.create`
Creates a new page in a specific parent (e.g., a database).
- **Key Inputs**: 
    - `parent`: (Optional) Database ID or Page ID.
    - `properties`: (Optional) JSON object of page properties.
    - `children`: (Optional) Array of page blocks.
- **Outputs**: 
    - `page`: The created page object.

### `databases.query`
Query a Notion database with filters and sorting.
- **Key Inputs**: `database_id`, `filter`, `sorts`.

### `pages.update`
Updates an existing page's properties.

### `search.post`
Searches for pages or databases in the workspace.

## Examples (WAML)

### Creating a Database Record
```waml
- step: add_log_entry
  call: notion.pages.create
  with:
    parent:
      database_id: "123456789"
    properties:
      Name:
        title:
          - text:
              content: "Log: " + inputs.event_name
      Status:
        select:
          name: "Active"
```

### Querying a Database
```waml
- step: find_tasks
  call: notion.databases.query
  with:
    database_id: "123456789"
    filter:
      property: "Status"
      select:
        equals: "Done"
```
```
