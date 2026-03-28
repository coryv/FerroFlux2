#!/usr/bin/env bash
# inventory.sh — List existing FerroFlux platform integrations and their nodes.
#
# Use this before expanding an existing platform to see what's already built
# and avoid duplication.
#
# Usage:
#   bash inventory.sh                  # List all platforms with node counts
#   bash inventory.sh <platform_id>    # List all nodes for a specific platform
#
# Examples:
#   bash inventory.sh
#   bash inventory.sh slack
#   bash inventory.sh stripe

set -uo pipefail

PLATFORMS_DIR="platforms"

if [[ ! -d "$PLATFORMS_DIR" ]]; then
  echo "ERROR: No 'platforms/' directory found. Run this from the FerroFlux2 project root." >&2
  exit 1
fi

# ── Helper: extract a YAML scalar value ────────────────────────────────────────

get_yaml_value() {
  local file="$1"
  local key="$2"
  grep -m1 "^  ${key}:" "$file" 2>/dev/null | sed "s/^  ${key}:[[:space:]]*//" | tr -d '"' | xargs
}

get_node_id() {
  local file="$1"
  grep -m1 "^  id:" "$file" 2>/dev/null | sed 's/^  id:[[:space:]]*//' | tr -d '"' | xargs
}

get_node_type() {
  local file="$1"
  grep -m1 "^  type:" "$file" 2>/dev/null | sed 's/^  type:[[:space:]]*//' | tr -d '"' | xargs
}

# ── Single platform detail view ────────────────────────────────────────────────

show_platform_detail() {
  local platform_id="$1"
  local platform_dir="$PLATFORMS_DIR/$platform_id"

  if [[ ! -d "$platform_dir" ]]; then
    echo "ERROR: Platform '$platform_id' not found in $PLATFORMS_DIR/" >&2
    exit 1
  fi

  local platform_file="$platform_dir/$platform_id.yaml"
  local platform_name="$platform_id"
  local base_url=""

  if [[ -f "$platform_file" ]]; then
    platform_name=$(get_yaml_value "$platform_file" "name")
    base_url=$(grep -m1 "base_url:" "$platform_file" 2>/dev/null | sed 's/.*base_url:[[:space:]]*//' | tr -d '"' | xargs)
  fi

  echo "$platform_name ($platform_id)"
  [[ -n "$base_url" ]] && echo "  Base URL: $base_url"
  echo ""

  local actions=()
  local triggers=()
  local other=()

  for file in "$platform_dir"/*.yaml; do
    [[ -f "$file" ]] || continue
    local basename
    basename=$(basename "$file")

    # Skip the platform definition file itself
    [[ "$basename" == "$platform_id.yaml" ]] && continue
    # Skip credentials file
    [[ "$basename" == "credentials.yaml" ]] && continue
    # Skip GAPS.md (not yaml, but just in case)
    [[ "$basename" == *.md ]] && continue

    local node_id
    node_id=$(get_node_id "$file")
    local node_type
    node_type=$(get_node_type "$file")

    case "$node_type" in
      Action)   actions+=("$node_id ($basename)") ;;
      Trigger)  triggers+=("$node_id ($basename)") ;;
      *)        other+=("$node_id ($basename) [type: $node_type]") ;;
    esac
  done

  local total=$(( ${#actions[@]} + ${#triggers[@]} + ${#other[@]} ))

  if [[ ${#actions[@]} -gt 0 ]]; then
    echo "  Actions (${#actions[@]}):"
    for a in "${actions[@]}"; do
      echo "    • $a"
    done
    echo ""
  fi

  if [[ ${#triggers[@]} -gt 0 ]]; then
    echo "  Triggers (${#triggers[@]}):"
    for t in "${triggers[@]}"; do
      echo "    • $t"
    done
    echo ""
  fi

  if [[ ${#other[@]} -gt 0 ]]; then
    echo "  Other (${#other[@]}):"
    for o in "${other[@]}"; do
      echo "    • $o"
    done
    echo ""
  fi

  if [[ $total -eq 0 ]]; then
    echo "  (no nodes yet — only platform definition exists)"
    echo ""
  fi

  echo "  Total: $total node(s)"

  # Show GAPS.md if it exists
  if [[ -f "$platform_dir/GAPS.md" ]]; then
    echo ""
    echo "  ⚠ GAPS.md exists — some nodes were intentionally omitted"
    grep "^## " "$platform_dir/GAPS.md" 2>/dev/null | sed 's/^## /    gap: /' || true
  fi
}

# ── All platforms summary view ─────────────────────────────────────────────────

show_all_platforms() {
  local platform_count=0
  local total_actions=0
  local total_triggers=0

  echo "FerroFlux Platform Inventory"
  echo "════════════════════════════"
  echo ""

  for platform_dir in "$PLATFORMS_DIR"/*/; do
    [[ -d "$platform_dir" ]] || continue
    local platform_id
    platform_id=$(basename "$platform_dir")
    local platform_file="$platform_dir/$platform_id.yaml"

    # Check for platform definition file
    local has_platform_def="✓"
    if [[ ! -f "$platform_file" ]]; then
      has_platform_def="✗ MISSING"
    fi

    # Get platform name
    local platform_name="$platform_id"
    if [[ -f "$platform_file" ]]; then
      platform_name=$(get_yaml_value "$platform_file" "name")
      [[ -z "$platform_name" ]] && platform_name="$platform_id"
    fi

    # Count actions and triggers
    local actions=0
    local triggers=0

    for file in "$platform_dir"/*.yaml; do
      [[ -f "$file" ]] || continue
      local basename
      basename=$(basename "$file")
      [[ "$basename" == "$platform_id.yaml" ]] && continue
      [[ "$basename" == "credentials.yaml" ]] && continue

      local node_type
      node_type=$(get_node_type "$file")
      case "$node_type" in
        Action)  actions=$((actions + 1)) ;;
        Trigger) triggers=$((triggers + 1)) ;;
      esac
    done

    local total=$((actions + triggers))
    local gaps_note=""
    [[ -f "$platform_dir/GAPS.md" ]] && gaps_note=" [has GAPS.md]"

    printf "  %-20s  %-25s  def:%-8s  %2d actions  %2d triggers%s\n" \
      "$platform_id" "$platform_name" "$has_platform_def" "$actions" "$triggers" "$gaps_note"

    platform_count=$((platform_count + 1))
    total_actions=$((total_actions + actions))
    total_triggers=$((total_triggers + triggers))
  done

  echo ""
  echo "  $platform_count platform(s) · $total_actions action(s) · $total_triggers trigger(s)"
  echo ""
  echo "Run 'bash inventory.sh <platform_id>' for per-platform node detail."
}

# ── Entry point ────────────────────────────────────────────────────────────────

if [[ $# -ge 1 ]]; then
  show_platform_detail "$1"
else
  show_all_platforms
fi
