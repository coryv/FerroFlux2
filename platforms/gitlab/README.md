# GitLab Integration Guide

Connects to the GitLab REST API to manage projects and issues.

## Setup & Authentication
1. Generate a Personal Access Token (PAT) in your [GitLab Profile Settings](https://gitlab.com/-/profile/personal_access_tokens).
2. Add the required GitLab scopes: `api`, `read_user`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_ACCESS_TOKEN`).
4. Set the `config.base_url` to `https://gitlab.com/api/v4`.

## Available Actions

### `issues.create`
Creates a new issue in a specific project.
- **Key Inputs**: 
    - `id`: Project ID or URL-encoded path (e.g., `12345`).
    - `title`: Issue title.
    - `description`: (Optional) Issue details.
- **Outputs**: 
    - `issue`: The created issue object.

## Examples (WAML)

### Creating a Bug
```waml
- step: report_issue
  call: gitlab.issues.create
  with:
    id: "98765"
    title: "Bug: Header is overlapping"
    description: "Found in v1.0.0"
```
```
