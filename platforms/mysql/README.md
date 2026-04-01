# MySQL Integration Guide

Connects to MySQL and MariaDB databases for structured data storage, querying, and transaction management.

## Setup & Authentication
1. Ensure your MySQL instance is accessible from the FerroFlux engine.
2. In FerroFlux, create a new Connection with the following configuration:
    - `host`: Database host.
    - `port`: Default is `3306`.
    - `user`: Database user.
    - `password`: Database password.
    - `database`: Name of the database.
    - `ssl`: (Optional) Set to `true` if required by your provider.

## Available Actions

### `query.execute`
Runs a raw SQL query with optional parameters using the `?` placeholder.
- **Key Inputs**: 
    - `sql`: The SQL statement (e.g., `SELECT * FROM users WHERE id = ?`).
    - `params`: (Optional) An array of values to bind to the query.
- **Outputs**: 
    - `rows`: The result set from the query.

### `rows.select`
A helper action to select rows without writing raw SQL.
- **Key Inputs**: 
    - `table`: Table name.
    - `columns`: (Optional) Array of columns to return.
    - `filter`: (Optional) A JSON object for basic filtering (e.g., `{ "status": "active" }`).

### `rows.insert`
Inserts one or more rows into a table.
- **Key Inputs**: 
    - `table`: Table name.
    - `data`: A JSON object or array of objects representing the rows to insert.

### `tx.begin` / `tx.commit` / `tx.rollback`
Used to manage database transactions for atomic operations.

## Examples (WAML)

### Executing a Parameterized Query
```waml
- step: get_user
  call: mysql.query.execute
  with:
    sql: "SELECT email, name FROM users WHERE id = ?"
    params: [inputs.user_id]
```

### Batch Inserting Records
```waml
- step: log_events
  call: mysql.rows.insert
  with:
    table: events
    data:
      - type: "login"
        user_id: "user_1"
      - type: "logout"
        user_id: "user_2"
```
