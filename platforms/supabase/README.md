# Supabase Integration Guide

Connects to the Supabase API to manage rows in PostgreSQL tables via the PostgREST interface.

## Setup & Authentication
1. Generate an API Key in the [Supabase Dashboard](https://app.supabase.com/).
2. In FerroFlux, create a new Connection and add the following:
    - `apiUrl`: `https://YOUR_PROJECT_REF.supabase.co/rest/v1`
    - `apiKey`: `YOUR_SUPABASE_ANON_KEY`
    - `Authorization`: `Bearer YOUR_SUPABASE_ANON_KEY`

## Available Actions

### `rows.select`
Selects rows from a table with filtering and sorting.
- **Key Inputs**: 
    - `table`: Table name.
    - `select`: (Optional) Comma-separated list of columns.
    - `filter`: (Optional) A JSON object for filtering (e.g., `{ "status": "active" }`).
- **Outputs**: 
    - `rows`: The result set as an array of objects.

### `rows.insert`
Inserts one or more rows into a table.
- **Key Inputs**: 
    - `table`: Table name.
    - `data`: A JSON object or array of objects representing the rows to insert.

### `rows.update`
Updates rows in a table matching a filter.
- **Key Inputs**: 
    - `table`, `data`, `filter`.

### `rows.delete`
Deletes rows from a table matching a filter.

## Examples (WAML)

### Selecting Active Rows
```waml
- step: get_active_tasks
  call: supabase.rows.select
  with:
    table: tasks
    select: "id,title,status"
    filter:
      status: "active"
```

### Batch Inserting Rows
```waml
- step: log_events
  call: supabase.rows.insert
  with:
    table: events
    data:
      - type: "login"
        user_id: "user_1"
      - type: "logout"
        user_id: "user_2"
```
```
