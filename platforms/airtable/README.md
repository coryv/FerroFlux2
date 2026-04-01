# Airtable Integration Guide

Connects to the Airtable API to manage records in bases and tables.

## Setup & Authentication
To use Airtable, you need a Personal Access Token (PAT).
1. Go to [Airtable Button](https://airtable.com/create/tokens) to create a token.
2. Ensure you grant the necessary scopes (e.g., `data.records:read`, `data.records:write`).
3. In FerroFlux, create a new Connection and paste your token into the `Authorization` header field (formatted as `Bearer YOUR_TOKEN`).

## Available Actions

### `records.create`
Creates one or more records in an Airtable base.
- **Key Inputs**: 
    - `base_id`: The ID of the base (found in API docs).
    - `table_id_or_name`: The ID or Name of the table.
    - `fields`: An object containing the field names and values.
- **Outputs**: 
    - `record`: The created record object.

### `records.get`
Retrieves a single record by its ID.
- **Key Inputs**: 
    - `base_id`, `table_id_or_name`, `record_id`.
- **Outputs**: 
    - `record`: The retrieved record object.

### `records.list`
Lists records in a table with pagination and filtering.
- **Key Inputs**: 
    - `base_id`, `table_id_or_name`.
    - `filterByFormula`: (Optional) Airtable formula for filtering.
    - `maxRecords`: (Optional) Limit the number of records.
- **Outputs**: 
    - `records`: An array of record objects.

### `records.update`
Updates a single record in a table (PATCH).
- **Key Inputs**: 
    - `base_id`, `table_id_or_name`, `record_id`, `fields`.
- **Outputs**: 
    - `record`: The updated record object.

### `records.delete`
Deletes a single record in a table.
- **Key Inputs**: 
    - `base_id`, `table_id_or_name`, `record_id`.

## Examples (WAML)

### Creating a Record
```waml
- step: add_customer
  call: airtable.records.create
  with:
    base_id: appXXXXXXXXXXXXXX
    table_id_or_name: Customers
    fields:
      Name: "Jane Doe"
      Email: "jane@example.com"
      Status: "Active"
```

### Listing Records with Filter
```waml
- step: find_active_tasks
  call: airtable.records.list
  with:
    base_id: appXXXXXXXXXXXXXX
    table_id_or_name: Tasks
    filterByFormula: "{Status} = 'Done'"
    maxRecords: 10
```
