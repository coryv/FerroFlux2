# Monday.com Integration Guide

Connects to the Monday.com API to manage boards, items, and columns.

## Setup & Authentication
1. Generate an API Token in your [Monday.com App Settings](https://app.monday.com/settings/apps).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_TOKEN`).

## Available Actions

### `items.create`
Creates a new item in a specific board and group.
- **Key Inputs**: 
    - `board_id`: Board ID.
    - `group_id`: (Optional) Group ID.
    - `item_name`: Item name.
    - `column_values`: (Optional) JSON string of column updates (e.g., `'{"status": "Done"}'`).
- **Outputs**: 
    - `item`: The created item object.

### `columns.update_value`
Updates a single column value for an item.
- **Key Inputs**: 
    - `board_id`, `item_id`, `column_id`, `value`.

### `items.add_update`
Adds a comment (update) to an item.

### `boards.list`
Lists all boards in the workspace.

## Available Triggers

### `items.new`
Fires when a new item is created in a board.
- **Settings**: `board_id`, `poll_interval`.

### `items.status_updated`
Fires when a status column changes on an item.

## Examples (WAML)

### Creating an Item
```waml
- step: create_order
  call: monday.items.create
  with:
    board_id: "123456789"
    item_name: "Order #1234"
    column_values: '{"status": "Success", "date": "2024-01-01"}'
```

### Updating a Column
```waml
- step: mark_as_shipped
  call: monday.columns.update_value
  with:
    board_id: "123456789"
    item_id: "987654321"
    column_id: "status"
    value: "Shipped"
```
```
