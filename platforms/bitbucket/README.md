# Bitbucket Integration Guide

Connects to the Bitbucket Cloud API to manage repositories, issues, and pull requests.

## Setup & Authentication
1. Generate an App Password in your [Bitbucket Profile Settings](https://bitbucket.org/account/settings/app-passwords/).
2. Add the required Bitbucket permissions: `repository`, `issue`.
3. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Basic BASE64(USERNAME:APP_PASSWORD)`).
4. Set the `config.base_url` to `https://api.bitbucket.org/2.0`.

## Available Actions

### `issues.create`
Creates a new issue in a specific repository.
- **Key Inputs**: 
    - `workspace`: Workspace ID or slug.
    - `repo_slug`: Repository slug.
    - `title`: Issue title.
    - `content`: (Optional) Issue details.
- **Outputs**: 
    - `issue`: The created issue object.

## Examples (WAML)

### Creating a Bug
```waml
- step: report_issue
  call: bitbucket.issues.create
  with:
    workspace: "my-team"
    repo_slug: "main-app"
    title: "Bug: Header is overlapping"
    content: "Found in v1.0.0"
```
```
