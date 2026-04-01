# Sentry Integration Guide

Connects to the Sentry REST API to manage issues, organizations, and alerts.

## Setup & Authentication
1. Generate an Internal Integration Token (PAT) in your [Sentry Settings](https://sentry.io/settings/account/api/auth-tokens/).
2. Add the required Sentry scopes: `event:read`, `issue:read`, `issue:write`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
4. Set the `config.base_url` to `https://sentry.io/api/0`.

## Available Actions

### `issues.list`
Lists issues (errors) in a specific project or organization.
- **Key Inputs**: 
    - `organization_slug`: (e.g., `my-company`).
    - `project_slug`: (e.g., `react-app`).
    - `query`: (Optional) Search query.
- **Outputs**: 
    - `issues`: An array of issue objects.

### `issues.get`
Retrieves a single issue by its ID.
- **Key Inputs**: `issue_id`.

### `issues.resolve`
Marks one or more issues as resolved.
- **Key Inputs**: `issue_id`.

## Examples (WAML)

### Listing Recent Errors
```waml
- step: get_errors
  call: sentry.issues.list
  with:
    organization_slug: "my-team"
    project_slug: "api-v1"
    query: "is:unresolved"
```

### Resolving a Specific Error
```waml
- step: fix_issue
  call: sentry.issues.resolve
  with:
    issue_id: "123456789"
```
```
