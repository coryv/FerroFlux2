# FerroFlux Tools Reference

**Tools** are the atomic building blocks of a Node's execution pipeline. They are stateless, Rust-implemented functions that perform specific tasks.

---

## `http_client`
Performs an HTTP request using `reqwest`.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `method` | String | GET, POST, PUT, DELETE, PATCH |
| `url` | String | The full URL target |
| `headers` | Object | (Optional) Key-value map of headers |
| `body` | String/Object | (Optional) Request body |

**Returns:**
- `status`: (Number) HTTP Status Code (e.g., 200, 404).
- `headers`: (Object) Response headers.
- `body`: (Value) Response body (parsed as JSON if possible, otherwise string).

**Example:**
```yaml
- id: call_api
  tool: http_client
  params:
    method: POST
    url: "https://api.site.com"
    body: 
      foo: bar
```

---

## `switch`
Evaluates a value against a set of cases and returns the matching output branch name. Useful for subsequent Routing logic.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `value` | Any | The value to test |
| `cases` | Array | List of case objects `{ "condition": "val", "output": "branch_name" }` |

**Returns:**
- `branch`: (String) The name of the output branch that matched.

**Example:**
```yaml
- id: check_status
  tool: switch
  params:
    value: "{{ steps.api.status }}"
    cases:
      - condition: "200"
        output: success
      - condition: default
        output: error
```

---

## `json_query`
Extracts a specific value from a JSON Object using a JSON Pointer path.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `json` | Object | The source JSON object (typically from `{{ steps.req.body }}`). |
| `path` | String | JSON Pointer path (e.g., `/users/0/name`). Must start with `/`. |

**Returns:**
- `result`: (Value) The extracted value, or `null` if not found.

---

## `emit`
Finalizes the node execution by sending data to a specific Output Port.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `port` | String | Name of the output port defined in `interface`. |
| `value` | Any | (Optional) The payload to send. |

**Returns:**
- None. (Effect is immediate emission).

---

## `logic`
Evaluates complex boolean logic against a data object. Supports nested AND/OR groups and various comparison operators.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `data` | Object | The data object to evaluate rules against. |
| `rules` | Array | List of Rule objects. First matching rule wins. |

**Rule Object Structure:**
- `output`: (String) The branch name to return if this rule matches.
- `condition`: (Object) The logical condition to evaluate.

**Condition Object:**
- `field`: (String) Key or JSON Pointer path to value in `data`.
- `operator`: (String) `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `starts_with`, `ends_with`.
- `value`: (Any) Target value to compare against.
- `operator` (Group): `AND` or `OR` (if using nested `rules`).

**Returns:**
- `match`: (String) The `output` name of the matching rule, or `"default"` if none match.

---

## `log`
Logs a message and optional data to the engine's tracing system.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `level` | String | INFO, WARN, ERROR, DEBUG (Default: INFO) |
| `message`| String | Descriptive message |
| `data`    | Any    | (Optional) Additional data to log |

---

## `math`
Performs basic arithmetic operations.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `a` | Number | First operand |
| `b` | Number | Second operand |
| `op` | String | `add`, `sub`, `mul`, `div` |

**Returns:**
- `result`: (Number) The calculation result.

---

## `sleep`
Pauses execution for a specified duration. Use sparingly!

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `duration_ms` | Number | Delay in milliseconds |

---

## `set_var` / `get_var`
Reads and writes to the global workflow memory (persists across nodes).

**Parameters (`set_var`):**
- `name`: (String) Variable name.
- `value`: (Any) Value to store.

**Parameters (`get_var`):**
- `name`: (String) Variable name.

**Returns (`get_var`):**
- `value`: (Any) The retrieved value.

---

## `rhai`
Executes an embedded Rhai script for complex transformations.

**Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `script` | String | The Rhai script to execute |
| `input`  | Any    | (Optional) Data available as `input` variable in script |

**Notes:** All local context variables are also injected into the Rhai scope.
