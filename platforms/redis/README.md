# Redis Integration Guide

Connects to a Redis instance for fast key-value storage, caching, and data structures.

## Setup & Authentication
1. Ensure your Redis instance is accessible from the FerroFlux engine.
2. In FerroFlux, create a new Connection with the following configuration:
    - `host`: Redis host.
    - `port`: Default is `6379`.
    - `password`: (Optional) Redis password.
    - `db`: (Optional) Redis database index.

## Available Actions

### `key.get` / `keys.get`
Retrieves one or more values from Redis keys.
- **Key Inputs**: 
    - `key`: The key name.
- **Outputs**: 
    - `value`: The retrieved value.

### `key.set` / `keys.set`
Stores one or more values in Redis keys.
- **Key Inputs**: 
    - `key`: The key name.
    - `value`: The value to store.

## Examples (WAML)

### Caching a Value
```waml
- step: cache_user_data
  call: redis.key.set
  with:
    key: "user:123"
    value: steps.get_user.data
```

### Retrieving a Cached Value
```waml
- step: load_user_data
  call: redis.key.get
  with:
    key: "user:123"
```
```
