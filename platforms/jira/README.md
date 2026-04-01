# Atlassian Jira Integration Guide

Connects to the Jira Software Cloud API to manage issues, projects, and sprints.

## Setup & Authentication
1. Generate an API Token in your [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(EMAIL:API_TOKEN)`).

## Available Actions

### `issues.create`
Creates a new issue in a specific project.
- **Key Inputs**: 
    - `project`: Project key or ID.
    - `summary`: Issue title.
    - `issuetype`: (e.g., `Task`, `Bug`, `Story`).
    - `description`: (Optional) Issue details.
- **Outputs**: 
    - `issue`: The created issue object.

### `issues.transition`
Changes the status of an issue (e.g., `To Do` -> `In Progress`).
- **Key Inputs**: 
    - `issue_id_or_key`: Issue key (e.g., `PROJ-123`).
    - `transition_id`: The ID of the transition to perform.

### `issues.search`
Search for issues using JQL (Jira Query Language).
- **Key Inputs**: `jql` (e.g., `project = PROJ AND status = TODO`).

### `users.get_by_email`
Retrieves a user's account ID by their email address.

## Available Triggers

### `issues.new`
Fires when a new issue is created in a project.
- **Settings**: `project_key`, `poll_interval`.

### `issues.updated`
Fires when an issue is modified (status, fields, etc.).

## Examples (WAML)

### Creating a Bug
```waml
- step: report_bug
  call: jira.issues.create
  with:
    project: "PLAT"
    summary: "UI: Login button is misaligned"
    issuetype: "Bug"
    description: "Issue found on mobile devices."
```

### Transitioning an Issue
```waml
- step: start_work
  call: jira.issues.transition
  with:
    issue_id_or_key: "PROJ-123"
    transition_id: "21" # ID for 'In Progress'
```
```
