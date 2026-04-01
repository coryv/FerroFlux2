# GitHub Integration Guide

Connects to the GitHub REST API to manage repositories, issues, and pull requests.

## Setup & Authentication
1. Generate a Personal Access Token (classic) or a Fine-grained Token in your [GitHub Developer Settings](https://github.com/settings/tokens).
2. Add the required GitHub scopes: `repo`, `read:user`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).

## Available Actions

### `issues.create`
Creates a new issue in a specific repository.
- **Key Inputs**: 
    - `owner`: Repository owner (e.g., `ferroflux-dev`).
    - `repo`: Repository name.
    - `title`: Issue title.
    - `body`: (Optional) Issue details.
- **Outputs**: 
    - `issue`: The created issue object.

### `repos.list`
Lists repositories for the authenticated user or an organization.
- **Key Inputs**: `org` (Optional).

### `issues.get`
Retrieves a single issue by its number.
- **Key Inputs**: `owner`, `repo`, `issue_number`.

## Examples (WAML)

### Creating a Bug Report
```waml
- step: report_bug
  call: github.issues.create
  with:
    owner: "my-team"
    repo: "main-app"
    title: "Bug: Sidebar is hidden"
    body: "Reported by user " + inputs.user_id
```

### Listing Repositories
```waml
- step: find_repos
  call: github.repos.list
  with:
    org: "ferroflux-dev"
```
```
