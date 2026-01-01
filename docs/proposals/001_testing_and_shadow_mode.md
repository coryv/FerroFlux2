# Proposal: Progressive Testing & Shadow Mode

## 1. Vision: The "Time Travel" Dev Loop

We aim to provide a safe, iterative, and "inside-out" testing experience that feels magical. Users should be able to:
1.  **Inspect** any past run with DVR-like precision.
2.  **Replay** any node instantly using historical inputs (no need to re-run the whole graph).
3.  **Simulate** dangerous actions safely using "Shadow Mode".

## 2. Core Concepts

### 2.1. Fork & Replay
Instead of re-executing an entire workflow to test the 5th step, we leverage our immutable `SecureTicket` architecture.
- **Action**: User clicks "Run Node" on Node 5.
- **Mechanism**: The engine retrieves the `OutputTicket` from Node 4 (from the last trace) and injects it into Node 5's `Inbox`.
- **Benfit**: Instant feedback loop.

### 2.2. Shadow Mode ("Safe Dry Run")
Shadow Mode allows a node to execute its *logic* (variable parsing, switches, loops) but **intercepts** external side effects (HTTP calls, DB writes).
- **Mocking**: Replaces dangerous tools (`http_client`, `ssh`) with `mock` versions that return user-defined or schema-compliant dummy data.
- **Verification**: Allows checking "What *would* this have sent to Slack?" without actually sending it.

## 3. SDK Enhancements (Rust Backend)

### 3.1. The `Shadow` Component
We will introduce a `Shadow` generic component or mode flag.

```rust
#[derive(Component)]
pub struct ShadowExecution {
    pub mocked_tools: HashMap<String, MockConfig>,
}

pub struct MockConfig {
    pub return_value: serde_json::Value,
    pub delay_ms: u64,
}
```

### 3.2. Tool Context Upgrade
The `ToolContext` will need to know if it's in Shadow Mode.
```rust
pub struct ToolContext<'a> {
    pub is_shadow_mode: bool,
    // ...
}
```
Standard tools like `http_client` will check this flag. If true, they log the intent ("Would have POSTed to X") and return the mock value from the registry instead of performing the network call.

### 3.3. API Expansion
New commands for the `ApiWorker`:
- `SimulateNode(NodeId, InputTicketId, MockConfig)`: Runs a node ephemerally, not affecting the main graph state, and returns the result directly to the caller.

## 4. UI Enhancements (Frontend)

### 4.1. "Live" Inspector Panel
- **History Scrubber**: A slider/list to jump between previous execution traces of the current workflow.
- **Data Viewer**: When a node is selected, show "Input Payload" and "Output Result" side-by-side.

### 4.2. The "Play" Button
- **Run Node**: A play button on every node.
  - *Hover State*: "Run using input from Trace #1234".
- **Mock Editor**: A small popover to define mock return values for Shadow Mode.

### 4.3. Visual Feedback
- **Shadow Mode Toggle**: A global or per-node toggle to enable Shadow Mode.
- **Ghost Styling**: Nodes running in Shadow Mode should have a distinct visual style (e.g., dashed border, ethereal glow) to differentiate from live production nodes.

## 5. Implementation Roadmap

1.  **Phase 1: Replay API** (Low Hanging Fruit)
    - Expose `get_latest_ticket(node_id)` API.
    - Connect UI "Run Node" button to `TriggerNode(node_id, latest_ticket)`.
2.  **Phase 2: Shadow Infrastructure**
    - Add `ShadowExecution` component.
    - Update `http_client` to respect shadow mode.
3.  **Phase 3: UI Integration**
    - Build the History Scrubber and Mock Configuration UI.
