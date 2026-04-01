# MongoDB Integration Guide

Connects to a MongoDB database to perform CRUD operations on document collections.

## Setup & Authentication
1. Ensure your MongoDB instance is accessible from the FerroFlux engine.
2. In FerroFlux, create a new Connection with the following configuration:
    - `connection_uri`: The connection string (e.g., `mongodb+srv://user:pass@cluster0.mongodb.net/`).
    - `database`: Name of the default database.

## Available Actions

### `document.find`
Retrieves a single document from a collection.
- **Key Inputs**: 
    - `collection`: Collection name.
    - `filter`: (Optional) A JSON object defining the search criteria.
- **Outputs**: 
    - `document`: The retrieved document.

### `documents.find`
Retrieves multiple documents from a collection.
- **Key Inputs**: 
    - `collection`: Collection name.
    - `filter`: (Optional) A JSON object defining the search criteria.
    - `limit`, `skip`.
- **Outputs**: 
    - `documents`: An array of documents.

### `document.insert` / `documents.insert`
Inserts one or more documents into a collection.
- **Key Inputs**: 
    - `collection`: Collection name.
    - `document` / `documents`: JSON objects.

### `document.update`
Updates a single document in a collection.
- **Key Inputs**: 
    - `collection`, `filter`, `update`.

### `document.delete`
Deletes a single document in a collection.

## Examples (WAML)

### Searching for a User Document
```waml
- step: find_user
  call: mongodb.document.find
  with:
    collection: "users"
    filter:
      email: "user@example.com"
```

### Updating a Record
```waml
- step: mark_as_active
  call: mongodb.document.update
  with:
    collection: "users"
    filter:
      email: "user@example.com"
    update:
      $set:
        status: "active"
```
```
