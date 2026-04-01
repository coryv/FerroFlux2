# ClickUp Integration Guide

Connects to the ClickUp API for managing lists, tasks, and team productivity.

## Setup & Authentication
1. Generate an API Token in your [ClickUp Personal Settings](https://app.clickup.com/settings/apps).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_TOKEN`).

## Available Actions

### `tasks.create`
Creates a new task in a specific ClickUp list.
- **Key Inputs**: 
    - `list_id`: List ID.
    - `name`: Task name.
    - `description`: (Optional) Task description.
    - `status`: (Optional) Task status.
- **Outputs**: 
    - `task`: The created task object.

### `tasks.update`
Updates an existing task's fields or status.
- **Key Inputs**: 
    - `task_id`: The ID of the task.
    - `data`: JSON object of fields to update.

### `lists.create`
Creates a new list in a specific folder.

### `tasks.add_comment`
Adds a comment to a task.

## Available Triggers

### `tasks.new`
Fires when a new task is created in a list.
- **Settings**: `list_id`, `poll_interval`.

### `tasks.status_updated`
Fires when a task's status changes.

## Examples (WAML)

### Creating a Task
```waml
- step: create_feature_request
  call: clickup.tasks.create
  with:
    list_id: "12345678"
    name: "Feature: Add Dark Mode"
    description: "Requested by multiple users."
```

### Changing Task Status
```waml
- step: move_to_testing
  call: clickup.tasks.update
  with:
    task_id: "862r3j1z"
    data:
      status: "testing"
```
```
