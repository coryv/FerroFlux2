# Vercel Integration Guide

Connects to the Vercel API for managing deployments, project configurations, and domain management.

## Setup & Authentication
1. Generate an API Token in your [Vercel Account Settings](https://vercel.com/account/tokens).
2. In FerroFlux, create a new Connection and add it to the `Authorization` header field (formatted as `Bearer YOUR_API_TOKEN`).
3. Set the `config.base_url` to `https://api.vercel.com`.

## Available Actions

### `deployments.list`
Lists deployments for the authenticated user or an organization.
- **Key Inputs**: 
    - `app`: (Optional) Project name or ID.
- **Outputs**: 
    - `deployments`: An array of deployment objects.

### `deployments.get`
Retrieves a single deployment by its ID or host.
- **Key Inputs**: `id`.

### `deployments.create`
Creates a new deployment for a project.

## Examples (WAML)

### Listing Recent Deployments
```waml
- step: get_deployments
  call: vercel.deployments.list
  with:
    app: "my-next-app"
```

### Retrieving Deployment Details
```waml
- step: check_status
  call: vercel.deployments.get
  with:
    id: "dpl_123456789"
```
```
