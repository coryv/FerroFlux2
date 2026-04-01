# Asana Integration Guide

Connects to the Asana API to manage projects, tasks, and team collaboration.

## Setup & Authentication
1. Generate a Personal Access Token (PAT) in your [Asana Developer Console](https://app.asana.com/0/developer-console).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `tasks.create`
Creates a new task in a specific project or workspace.
- **Key Inputs**: 
    - `workspace`: Workspace ID.
    - `projects`: Array of project IDs.
    - `name`: Task name.
    - `notes`: (Optional) Task description.
- **Outputs**: 
    - `task`: The created task object.

### `tasks.update`
Updates an existing task's fields or status.
- **Key Inputs**: 
    - `task_gid`: The global ID of the task.
    - `data`: JSON object of fields to update (e.g., `{ "completed": true }`).

### `projects.list`
Lists all projects in a workspace.

### `comments.create`
Adds a comment (story) to a task.

## Examples (WAML)

### Creating a Task
```waml
- step: create_bug_report
  call: asana.tasks.create
  with:
    workspace: "123456789"
    projects: ["987654321"]
    name: "Bug: Login fails on Safari"
    notes: "Reported by user Jane Doe."
```

### Completing a Task
```waml
- step: complete_task
  call: asana.tasks.update
  with:
    task_gid: "1122334455"
    data:
      completed: true
```
```
