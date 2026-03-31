#!/usr/bin/env bash
# pre-lint.sh — Fast static analysis for FerroFlux integration YAML files.
#
# Catches template syntax bugs and common authoring mistakes. Run this BEFORE
# the slower Rust validator to get faster feedback on the most common errors.
#
# Most importantly: catches leftover Handlebars {{ }} syntax (now invalid — use CEL)
# and invalid platform property access, which the Rust validator does NOT detect.
#
# Usage:
#   bash pre-lint.sh <directory>
#
# Examples:
#   bash pre-lint.sh platforms/slack/
#   bash pre-lint.sh platforms/stripe/
#
# Exit codes:
#   0 — No errors (warnings may still be present)
#   1 — One or more errors found

set -uo pipefail

# ── Argument validation ────────────────────────────────────────────────────────

if [[ $# -lt 1 ]]; then
  echo "Usage: bash pre-lint.sh <directory>" >&2
  exit 1
fi

TARGET="$1"

if [[ ! -d "$TARGET" ]]; then
  echo "ERROR: Directory '$TARGET' does not exist." >&2
  exit 1
fi

# ── State ──────────────────────────────────────────────────────────────────────

TOTAL_ERRORS=0
TOTAL_WARNINGS=0
CHECKED=0

# ── Per-file checks ────────────────────────────────────────────────────────────

check_file() {
  local file="$1"
  local file_errors=0
  local file_warnings=0
  local file_header_printed=0

  print_header() {
    if [[ $file_header_printed -eq 0 ]]; then
      echo "$file"
      file_header_printed=1
    fi
  }

  emit_error() {
    print_header
    echo "  ERROR [$1] $2"
    file_errors=$((file_errors + 1))
  }

  emit_warn() {
    print_header
    echo "  WARN  [$1] $2"
    file_warnings=$((file_warnings + 1))
  }

  # ── Check 1: Leftover Handlebars syntax ───────────────────────────────────
  # FerroFlux uses CEL expressions — Handlebars {{ }} is no longer supported.
  # Examples of wrong syntax and their CEL equivalents:
  #   {{ platform.base_url }}/path  →  platform.base_url + "/path"
  #   {{ get 'inputs.field' }}      →  inputs.field
  #   {{ get 'settings.x' }}        →  settings.x
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    emit_error "handlebars-syntax" "$(echo "$line" | xargs) — use CEL expressions instead of {{ }}; e.g. 'inputs.field', 'settings.field', 'platform.base_url + \"/path\"'"
  done < <(grep -nE '\{\{' "$file" 2>/dev/null || true)

  # ── Check 2: http_client step without returns block ───────────────────────
  # The Rust validator catches this too, but catching it here is faster.
  if grep -q 'tool: http_client' "$file" 2>/dev/null; then
    if ! grep -q 'returns:' "$file" 2>/dev/null; then
      emit_error "missing-returns" "http_client step found but no 'returns:' block — add: returns: { status: status_code, body: response_body }"
    fi
  fi

  # ── Check 4: Hardcoded base URL (not using platform.base_url) ─────────────
  # The Rust validator emits this as a warning. We emit it here too for speed.
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    # Only flag url: lines, not base_url: in platform files
    if echo "$line" | grep -qE '^\s+base_url:'; then continue; fi
    if echo "$line" | grep -q 'platform\.base_url'; then continue; fi
    emit_warn "hardcoded-url" "$(echo "$line" | xargs) — use 'platform.base_url + \"/path\"' instead of a hardcoded URL"
  done < <(grep -nE '^\s+url:\s+"https?://' "$file" 2>/dev/null || true)

  # ── Check 5: Trailing slash on base_url ───────────────────────────────────
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    emit_warn "trailing-slash" "$(echo "$line" | xargs) — remove trailing slash from base_url"
  done < <(grep -nE '^\s+base_url:\s+".*/"' "$file" 2>/dev/null || true)

  # ── Check 6: Unfilled TODO placeholders ───────────────────────────────────
  # Scaffolded nodes have /TODO in URLs. Catch any that weren't replaced.
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    emit_warn "unfilled-todo" "$(echo "$line" | xargs) — replace the TODO placeholder with the real API path"
  done < <(grep -nE '/TODO\b' "$file" 2>/dev/null || true)

  # ── Check 7: invalid platform property access ─────────────────────────────
  # Only platform.base_url and platform.headers are valid in CEL expressions.
  # platform.api_key / platform.token / platform.secret do not exist — credentials
  # are pre-built into platform.headers by the engine. This is a hard runtime error.
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    emit_error "invalid-platform-property" "$(echo "$line" | xargs) — only platform.base_url and platform.headers are valid; credentials are already in platform.headers"
  done < <(grep -nE 'platform\.(api_key|token|secret|key|credentials)\b' "$file" 2>/dev/null || true)

  # ── Accumulate totals ──────────────────────────────────────────────────────
  TOTAL_ERRORS=$((TOTAL_ERRORS + file_errors))
  TOTAL_WARNINGS=$((TOTAL_WARNINGS + file_warnings))
  [[ $file_header_printed -eq 1 ]] && echo ""
}

# ── Process all YAML files ─────────────────────────────────────────────────────

for file in "$TARGET"/*.yaml; do
  [[ -f "$file" ]] || continue
  check_file "$file"
  CHECKED=$((CHECKED + 1))
done

if [[ $CHECKED -eq 0 ]]; then
  echo "No YAML files found in '$TARGET'" >&2
  exit 1
fi

# ── Summary ────────────────────────────────────────────────────────────────────

if [[ $TOTAL_ERRORS -eq 0 && $TOTAL_WARNINGS -eq 0 ]]; then
  echo "$CHECKED file(s) checked — no issues found"
else
  echo "$CHECKED file(s) checked, $TOTAL_ERRORS error(s), $TOTAL_WARNINGS warning(s)"
fi

# Exit 1 if any errors (warnings are OK)
[[ $TOTAL_ERRORS -eq 0 ]]
