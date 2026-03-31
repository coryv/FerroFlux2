#!/usr/bin/env bash
# scaffold-node.sh — Generate a structurally-valid FerroFlux node YAML skeleton.
#
# Usage:
#   bash scaffold-node.sh <platform_id> <node_type> <category> <verb> <node_name>
#
# Arguments:
#   platform_id   Platform this node belongs to (must match an existing platforms/<id>/ dir)
#   node_type     action | trigger
#   category      Lowercase category (e.g. messages, issues, payments, users)
#   verb          Lowercase verb or event name (e.g. send, create, list, new, completed)
#   node_name     Human-readable name (e.g. "Send Message", "New Issue", "Create Payment")
#
# Output:
#   platforms/<platform_id>/action.<category>.<verb>.yaml   (for action)
#   platforms/<platform_id>/trigger.<category>.<verb>.yaml  (for trigger)
#
# Examples:
#   bash scaffold-node.sh slack action messages send "Send Message"
#   bash scaffold-node.sh slack trigger messages new "New Message in Channel"
#   bash scaffold-node.sh stripe action payments create "Create Payment Intent"
#   bash scaffold-node.sh stripe trigger payments succeeded "Payment Succeeded"

set -euo pipefail

# ── Argument validation ────────────────────────────────────────────────────────

if [[ $# -lt 5 ]]; then
  echo "Usage: bash scaffold-node.sh <platform_id> <node_type> <category> <verb> <node_name>" >&2
  echo "" >&2
  echo "  node_type: action | trigger" >&2
  exit 1
fi

PLATFORM_ID="$1"
NODE_TYPE="$2"
CATEGORY="$3"
VERB="$4"
NODE_NAME="$5"

if [[ ! "$NODE_TYPE" =~ ^(action|trigger)$ ]]; then
  echo "ERROR: node_type must be 'action' or 'trigger', got '$NODE_TYPE'" >&2
  exit 1
fi

if [[ ! "$PLATFORM_ID" =~ ^[a-z0-9][a-z0-9_-]*$ ]]; then
  echo "ERROR: platform_id must be lowercase alphanumeric: '$PLATFORM_ID'" >&2
  exit 1
fi

if [[ ! "$CATEGORY" =~ ^[a-z0-9][a-z0-9_-]*$ ]]; then
  echo "ERROR: category must be lowercase alphanumeric (hyphens/underscores ok): '$CATEGORY'" >&2
  exit 1
fi

if [[ ! "$VERB" =~ ^[a-z0-9][a-z0-9_./-]*$ ]]; then
  echo "ERROR: verb must be lowercase alphanumeric (dots, hyphens ok): '$VERB'" >&2
  exit 1
fi

# ── Derived values ─────────────────────────────────────────────────────────────

# Capitalize first letter of category for the YAML display name
# (uses portable approach — ${VAR^} requires bash 4+, unavailable on macOS default bash)
_FIRST=$(echo "$CATEGORY" | cut -c1 | tr '[:lower:]' '[:upper:]')
_REST=$(echo "$CATEGORY" | cut -c2-)
CATEGORY_DISPLAY="${_FIRST}${_REST}"

# ── Path setup ─────────────────────────────────────────────────────────────────

PLATFORM_DIR="platforms/$PLATFORM_ID"

if [[ ! -d "$PLATFORM_DIR" ]]; then
  echo "ERROR: Platform directory '$PLATFORM_DIR' does not exist." >&2
  echo "       Create the platform first with scaffold-platform.sh" >&2
  exit 1
fi

NODE_ID="${PLATFORM_ID}.${CATEGORY}.${VERB}"
OUT_FILE="$PLATFORM_DIR/${NODE_TYPE}.${CATEGORY}.${VERB}.yaml"

if [[ -f "$OUT_FILE" ]]; then
  echo "ERROR: $OUT_FILE already exists. Delete it first if you want to regenerate." >&2
  exit 1
fi

# ── Generate content ───────────────────────────────────────────────────────────

if [[ "$NODE_TYPE" == "action" ]]; then
  cat > "$OUT_FILE" << YAML
meta:
  id: ${NODE_ID}
  name: ${NODE_NAME}
  category: ${CATEGORY_DISPLAY}
  type: Action
  platform: ${PLATFORM_ID}
  description: "TODO: Describe what this node does."
  version: "1.0.0"

interface:
  inputs:
    - name: Exec
      type: flow
    # TODO: add runtime inputs that get wired from previous nodes
    # - name: field_name
    #   type: string
  outputs:
    - name: Success
      type: flow
    - name: Error
      type: flow
    - name: result
      type: object
  # settings:
  #   - name: example_setting
  #     label: "Example Setting"
  #     type: string
  #     required: false
  #     placeholder: "default value"

execution:
  - id: request
    tool: http_client
    params:
      method: GET            # TODO: GET | POST | PUT | DELETE | PATCH
      url: platform.base_url + "/TODO"
      headers: platform.headers
      # body:                # Uncomment for POST/PUT/PATCH
      #   field: inputs.field_name
      #   setting: settings.example_setting
    returns:
      status: status_code
      body: response_body

  - id: check_status
    tool: switch
    params:
      value: steps.request.status_code
      cases:
        - condition: "200"   # TODO: add all success codes (201, 202, 204, etc.)
          output: success
        - condition: default
          output: error

routing:
  match: steps.check_status.branch
  cases:
    success:
      - tool: emit
        params:
          port: Success
      - tool: emit
        params:
          port: result
          value: steps.request.response_body
    error:
      - tool: emit
        params:
          port: Error
          value: steps.request.response_body
YAML

elif [[ "$NODE_TYPE" == "trigger" ]]; then
  cat > "$OUT_FILE" << YAML
meta:
  id: ${NODE_ID}
  name: ${NODE_NAME}
  category: ${CATEGORY_DISPLAY}
  type: Trigger
  platform: ${PLATFORM_ID}
  description: "TODO: Describe when this trigger fires."
  version: "1.0.0"

interface:
  inputs: []
  outputs:
    - name: Success
      type: flow
    - name: result
      type: object
  settings:
    - name: poll_interval
      label: "Poll Interval (minutes)"
      type: number
      required: false
      default: 1
      min: 1

execution:
  - id: request
    tool: http_client
    params:
      method: GET
      url: platform.base_url + "/TODO"
      headers: platform.headers
      query:
        since: event.cursor   # Stateful cursor for incremental polling
        limit: 100
        # TODO: add any filter/scope query params (channel_id, resource_id, etc.)
    returns:
      status: status_code
      body: response_body

routing:
  match: steps.request.status_code
  cases:
    "200":
      - tool: emit
        params:
          port: Success
      - tool: emit
        params:
          port: result
          value: steps.request.response_body
YAML
fi

# ── Output ─────────────────────────────────────────────────────────────────────

echo "✓ Created: $OUT_FILE"
echo ""
echo "TODOs to fill in:"
echo "  1. Replace the URL placeholder: platform.base_url + \"/TODO\""
if [[ "$NODE_TYPE" == "action" ]]; then
  echo "  2. Add inputs to interface.inputs (remove the # from example lines)"
  echo "  3. Add any settings (static config set once in UI)"
  echo "  4. Fill in the body fields for POST/PUT/PATCH"
  echo "  5. Add all valid HTTP success codes (201, 202, 204, etc.) to check_status"
  echo "  6. Extract specific fields from response_body using json_query (optional)"
elif [[ "$NODE_TYPE" == "trigger" ]]; then
  echo "  2. Add filter/scope settings (e.g. channel_id, repo, resource)"
  echo "  3. Adjust the cursor param name ('since', 'oldest', 'after', etc.) to match the API"
  echo "  4. Extract specific fields from response_body using json_query (optional)"
fi
echo ""
echo "Run pre-lint before validating:"
echo "  bash .claude/skills/ferroflux-integration/scripts/pre-lint.sh platforms/$PLATFORM_ID/"
