import os
import re

dirs_to_scan = ["platforms", "crates/ferroflux-testing/fixtures"]
base_known_vars = ["inputs", "settings", "platform", "event", "store", "context", "steps", "request", "response", "body", "headers", "query"]
# Structural keys only. Parameters like 'operation' or 'method' can be dynamic expressions.
id_keys = {"id", "type", "platform", "category", "source_id", "target_id", "source_handle", "target_handle", "port", "version"}
protected_literals = {"query", "merge", "flatten", "add", "sub", "mul", "div", "GET", "POST", "PUT", "DELETE", "PATCH", "INFO", "WARN", "ERROR", "Success", "True", "False"}

def is_safe_identifier(val: str) -> bool:
    if not val: return False
    if re.match(r'^[a-zA-Z_][a-zA-Z0-9_\-\.]*$', val):
        if val.lower() in ["true", "false", "null"]:
            return False
        return True
    return False

def transform_legacy_syntax(val: str) -> str:
    get_pattern = r"\{\{\s*get\s*['\"](.+?)['\"]\s*\}\}"
    def get_repl(m):
        path = m.group(1)
        if path.startswith("settings.") or path.startswith("inputs.") or path.startswith("steps.") or path.startswith("platform."):
            return path
        if path.startswith("body.") or path.startswith("headers.") or path.startswith("query."):
            return "inputs." + path
        return "inputs." + path
    val = re.sub(get_pattern, get_repl, val)

    template_pattern = r"\$\{(.+?)\}"
    matches = list(re.finditer(template_pattern, val))
    if not matches: return val
    if len(matches) == 1 and matches[0].group(0).strip() == val.strip():
        return matches[0].group(1)

    parts = []
    last_end = 0
    for m in matches:
        literal = val[last_end:m.start()]
        if literal: parts.append(f'"{literal}"')
        parts.append(m.group(1))
        last_end = m.end()
    literal = val[last_end:]
    if literal: parts.append(f'"{literal}"')
    return " + ".join(parts)

def is_intended_expression(val: str, key: str, all_vars: list) -> bool:
    if key in id_keys: return False
    if val in protected_literals: return False
    if val.lower() in ["true", "false"]: return False
    # Common triggers for CEL
    if "." in val or "(" in val or any(op in val for op in [" + ", " == ", " != ", " > ", " < ", " && ", " || "]):
        # Double check it's not a generic name. If it starts with a known root or node id, it's an expression.
        for root in ["inputs", "settings", "platform", "steps", "event", "body", "headers", "query"]:
            if val.startswith(root + ".") or val == root:
                return True
        for nid in all_vars:
            if val.startswith(nid + "."):
                return True
        # If it contains standard operators, it's likely an expression
        if any(op in val for op in [" + ", " == ", " != ", " > ", " < ", " && ", " || "]):
            return True
    return False

def format_yaml_val(val: str) -> str:
    if '"' in val and "'" not in val:
        return f"'{val}'"
    if any(c in val for c in [":", ">", "<", "[", "]", "{", "}", "#", "!", "&", "*", "|", "?", "\""]):
        if '"' in val:
            if "'" not in val: return f"'{val}'"
            else: return f'"{val.replace('"', '\\"')}"'
        return f'"{val}"'
    return val

def migrate_file(filepath):
    with open(filepath, "r") as f:
        lines = f.readlines()
    changed = False
    new_lines = []
    node_ids = set()
    for line in lines:
        if "id:" in line and not line.strip().startswith("#"):
            parts = line.split("id:", 1)
            if "id" in parts[0] or parts[0].strip() == "-":
                nid = parts[1].strip().strip('"').strip("'")
                if nid and is_safe_identifier(nid): node_ids.add(nid)
    all_vars = base_known_vars + list(node_ids)
    for line in lines:
        original = line
        if ":" in line and not line.strip().startswith("#"):
            left_part, right_part = line.split(":", 1)
            key_match = re.match(r'^(\s*(?:-\s+)?)([\w\.-]+)(\s*)$', left_part)
            if not key_match:
                new_lines.append(line)
                continue
            prefix, key, suffix = key_match.groups()
            raw_val = right_part.strip()
            comment = ""
            if " #" in right_part:
                raw_val, comment = right_part.split(" #", 1)
                raw_val = raw_val.strip()
                comment = " #" + comment.rstrip()
            if not raw_val or raw_val.startswith("[") or raw_val.startswith("{") or raw_val.startswith(">") or raw_val.startswith("|"):
                new_lines.append(line)
                continue
            is_quoted = (raw_val.startswith('"') and raw_val.endswith('"')) or (raw_val.startswith("'") and raw_val.endswith("'"))
            inner_val = raw_val[1:-1] if is_quoted else raw_val
            
            # De-template
            transformed = transform_legacy_syntax(inner_val)
            if transformed != inner_val:
                final_val = format_yaml_val("=" + transformed)
            else:
                if inner_val.startswith("="): inner_val = inner_val[1:]
                if key in id_keys:
                    final_val = inner_val if is_safe_identifier(inner_val) else format_yaml_val(inner_val)
                elif is_intended_expression(inner_val, key, all_vars):
                    final_val = format_yaml_val("=" + inner_val)
                else:
                    final_val = inner_val if is_safe_identifier(inner_val) else format_yaml_val(inner_val)
            line = f"{left_part}:{right_part.replace(right_part.strip(), final_val, 1)}".rstrip() + comment + "\n"
        if line != original: changed = True
        new_lines.append(line)
    if changed:
        with open(filepath, "w") as f: f.writelines(new_lines)
        print(f"Migrated: {filepath}")

if __name__ == "__main__":
    count = 0
    for d in dirs_to_scan:
        for root, dirs, files in os.walk(d):
            for f in files:
                if f.endswith(".yaml") or f.endswith(".waml"):
                    migrate_file(os.path.join(root, f))
                    count += 1
    print(f"Checked {count} files.")
