#!/usr/bin/env bash
# scaffold-platform.sh — Generate a valid FerroFlux platform YAML skeleton.
#
# Usage:
#   bash scaffold-platform.sh <platform_id> <platform_name> <base_url> <auth_type> [category]
#
# Arguments:
#   platform_id    Short lowercase identifier (e.g. stripe, hubspot, google-calendar)
#   platform_name  Human-readable name (e.g. "Stripe", "HubSpot")
#   base_url       API base URL, no trailing slash (e.g. https://api.stripe.com/v1)
#   auth_type      One of: bearer | custom:<header-name> | oauth2 | none
#   category       Optional. Defaults to "Integration"
#                  (e.g. Communication, Developer Tools, AI/ML, E-Commerce, Databases)
#
# Auth type reference:
#   bearer                  → Authorization: Bearer <token>  (most common)
#   custom:x-goog-api-key   → Custom header with API key (Gemini, some others)
#   custom:api-key          → Custom header (Azure OpenAI style)
#   oauth2                  → connection_select setting; token injected at runtime
#   none                    → No auth (public APIs like Open-Meteo)
#
# Examples:
#   bash scaffold-platform.sh stripe "Stripe" "https://api.stripe.com/v1" bearer "E-Commerce"
#   bash scaffold-platform.sh gemini "Google Gemini" "https://generativelanguage.googleapis.com" "custom:x-goog-api-key" "AI/ML"
#   bash scaffold-platform.sh google-calendar "Google Calendar" "https://www.googleapis.com/calendar/v3" oauth2 "Calendar"
#   bash scaffold-platform.sh open-meteo "Open-Meteo" "https://api.open-meteo.com/v1" none "Weather"

set -euo pipefail

# ── Argument validation ────────────────────────────────────────────────────────

if [[ $# -lt 4 ]]; then
  echo "Usage: bash scaffold-platform.sh <platform_id> <platform_name> <base_url> <auth_type> [category]" >&2
  echo "" >&2
  echo "  auth_type: bearer | custom:<header-name> | oauth2 | none" >&2
  exit 1
fi

PLATFORM_ID="$1"
PLATFORM_NAME="$2"
BASE_URL="$3"
AUTH_TYPE="$4"
CATEGORY="${5:-Integration}"

# Validate platform_id format
if [[ ! "$PLATFORM_ID" =~ ^[a-z0-9][a-z0-9_-]*$ ]]; then
  echo "ERROR: platform_id must be lowercase alphanumeric (hyphens and underscores allowed): '$PLATFORM_ID'" >&2
  exit 1
fi

# Validate base_url has no trailing slash
if [[ "$BASE_URL" =~ /$ ]]; then
  echo "ERROR: base_url must not have a trailing slash: '$BASE_URL'" >&2
  exit 1
fi

# Validate base_url starts with https://
if [[ ! "$BASE_URL" =~ ^https?:// ]]; then
  echo "ERROR: base_url must start with https:// (or http://): '$BASE_URL'" >&2
  exit 1
fi

# ── Path setup ─────────────────────────────────────────────────────────────────

OUT_DIR="platforms/$PLATFORM_ID"
OUT_FILE="$OUT_DIR/$PLATFORM_ID.yaml"

mkdir -p "$OUT_DIR"

if [[ -f "$OUT_FILE" ]]; then
  echo "ERROR: $OUT_FILE already exists. Delete it first if you want to regenerate." >&2
  exit 1
fi

# ── Generate content ───────────────────────────────────────────────────────────

{
  echo "meta:"
  echo "  id: $PLATFORM_ID"
  echo "  name: $PLATFORM_NAME"
  echo "  type: Platform"
  echo "  category: $CATEGORY"
  echo "  description: \"Connects to the $PLATFORM_NAME API.\""
  echo "  version: \"1.0.0\""
  echo ""
  echo "config:"
  echo "  base_url: \"$BASE_URL\""
  echo "  headers:"
  echo "    Content-Type: \"application/json\""

  case "$AUTH_TYPE" in
    bearer)
      echo "    Authorization: \"Bearer PASTE_YOUR_KEY_HERE\""
      echo ""
      echo "settings:"
      echo "  - name: api_key"
      echo "    label: \"API Key\""
      echo "    type: string"
      echo "    required: true"
      ;;

    custom:*)
      HEADER_NAME="${AUTH_TYPE#custom:}"
      if [[ -z "$HEADER_NAME" ]]; then
        echo "ERROR: auth_type 'custom:' requires a header name, e.g. custom:x-goog-api-key" >&2
        exit 1
      fi
      echo "    $HEADER_NAME: \"PASTE_YOUR_KEY_HERE\""
      echo ""
      echo "settings:"
      echo "  - name: api_key"
      echo "    label: \"API Key\""
      echo "    type: string"
      echo "    required: true"
      ;;

    oauth2)
      echo "    # No Authorization header — injected automatically from the stored connection"
      echo ""
      echo "settings:"
      echo "  - name: connection"
      echo "    label: \"$PLATFORM_NAME Account\""
      echo "    type: connection_select"
      echo "    required: true"
      ;;

    none)
      # No auth — nothing to add
      ;;

    *)
      echo "ERROR: Unknown auth_type '$AUTH_TYPE'. Use: bearer | custom:<header-name> | oauth2 | none" >&2
      exit 1
      ;;
  esac
} > "$OUT_FILE"

# ── Output ─────────────────────────────────────────────────────────────────────

echo "✓ Created: $OUT_FILE"
echo ""
echo "Next steps:"
echo "  1. Verify the base_url and auth pattern look correct"
echo "  2. Add any additional platform-level headers if the API requires them"
echo "  3. Run: bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh $PLATFORM_ID action <category> <verb> \"<Node Name>\""
