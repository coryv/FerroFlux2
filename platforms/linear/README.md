# Linear Integration Guide

Connects to the Linear API for high-performance issue tracking and project management.

## Setup & Authentication
1. Generate a Personal API Key in your [Linear Settings](https://linear.app/settings/api).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_KEY`).

## Available Actions

### `issues.create`
Creates a new issue in a specific team.
- **Key Inputs**: 
    - `teamId`: Team ID.
    - `title`: Issue title.
    - `description`: (Optional) Issue details.
    - `priority`: (Optional) 0 (None), 1 (Urgent), 2 (High), 3 (Normal), 4 (Low).
- **Outputs**: 
    - `issue`: The created issue object.

### `issues.update`
Updates an existing issue's fields or status.
- **Key Inputs**: 
    - `issueId`: The ID of the issue.
    - `data`: JSON object of fields to update.

### `teams.list`
Lists all teams in the workspace.

### `comments.create`
Adds a comment to an issue.

## Examples (WAML)

### Creating an Issue
```waml
- step: create_bug
  call: linear.issues.create
  with:
    teamId: "123-abc-456"
    title: "Bug: App crashes on export"
    description: "Found in v1.2.0"
    priority: 1
```

### Adding a Comment
```waml
- step: add_note
  call: linear.comments.create
  with:
    issueId: "ISS-123"
    body: "Investigating the logs now."
```
```
