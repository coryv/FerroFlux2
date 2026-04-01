# Salesforce Integration Guide

Connects to the Salesforce REST API to manage objects, records, and files in your Salesforce instance.

## Setup & Authentication
1. Generate an OAuth2 Client ID and Client Secret in your [Salesforce Connected Apps](https://help.salesforce.com/s/articleView?id=sf.connected_app_create.htm).
2. Add the required Salesforce permissions: `api`, `web`, `refreshToken`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `records.create`
Creates a new record in a specific Salesforce object.
- **Key Inputs**: 
    - `object`: Salesforce object name (e.g., `Account`, `Contact`, `Opportunity`).
    - `fields`: A JSON object of field names and values.
- **Outputs**: 
    - `id`: The ID of the created record.

### `records.query`
Executes a SOQL query to search for records.
- **Key Inputs**: `q` (SOQL query string).
- **Outputs**: 
    - `records`: An array of record objects.

### `records.update` / `records.delete`
Updates or deletes an existing record.

### `files.upload`
Uploads a file to Salesforce and associates it with a record.

## Examples (WAML)

### Creating an Account
```waml
- step: new_account
  call: salesforce.records.create
  with:
    object: "Account"
    fields:
      Name: "New Client - " + inputs.company_name
      Website: inputs.website
```

### Searching for Contacts
```waml
- step: find_contacts
  call: salesforce.records.query
  with:
    q: "SELECT Id, Name, Email FROM Contact WHERE Email = '" + inputs.email + "'"
```
```
