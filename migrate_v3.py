import os
import re

# Standard roots that indicate a CEL expression
base_known_vars = ["inputs", "settings", "platform", "steps", "event", "body", "headers", "query", "self"]

# Structural keys that should ALMOST ALWAYS be literals (mostly metadata)
id_keys = {
    "id", "type", "platform", "category", "source_id", "target_id", 
    "source_handle", "target_handle", "version", "name", 
    "label", "placeholder", "required", "default", 
    "action", "trigger", "tool", "call", "port"
}

# Values that are clearly literals even if they look like variables
protected_literals = {"query", "merge", "flatten", "add", "sub", "mul", "div", "GET", "POST", "PUT", "DELETE", "PATCH", "INFO", "WARN", "ERROR", "Success", "True", "False", "results", "result"}

def is_safe_identifier(s):
    return re.match(r'^[a-zA-Z_][a-zA-Z0-9_\.]*$', s) is not None

def is_intended_expression(val, all_vars):
    if not val: return False
    if val.startswith("="): return True
    
    # Clearly a complex expression
    if "(" in val or ")" in val or "+" in val or "-" in val or "*" in val or "/" in val or "==" in val or "||" in val or "&&" in val or "?" in val:
        return True
    
    # JSON or List
    if val.startswith("[") or val.startswith("{"): return False
    
    # Path or variable access
    for root in base_known_vars:
        if val.startswith(root + ".") or val == root: return True
        
    for nid in all_vars:
        if val.startswith(nid + ".") or val == nid: return True
            
    return False

def escape_val(val):
    if not val: return val
    if any(c in val for c in [":", ">", "<", "[", "]", "{", "}", "#", "!", "&", "*", "|", "?", "\""]):
        if '"' in val:
            if "'" not in val: return f"'{val}'"
            else: 
                v = val.replace('"', '\\"')
                return f'"{v}"'
        return f'"{val}"'
    return val

def get_node_ids(lines):
    node_ids = set()
    in_returns = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("id:"):
            parts = line.split(":", 1)
            if len(parts) > 1:
                nid = parts[1].strip().strip('"').strip("'")
                if nid and is_safe_identifier(nid): node_ids.add(nid)
        elif stripped.startswith("returns:"):
            in_returns = True
        elif in_returns:
            if ":" in line:
                if not line.startswith(" ") and not line.strip().startswith("-"):
                    in_returns = False
                else:
                    parts = line.split(":", 1)
                    if len(parts) > 1:
                        var_name = parts[1].strip().strip('"').strip("'")
                        if var_name and is_safe_identifier(var_name): node_ids.add(var_name)
            elif stripped == "" or stripped.startswith("#"):
                pass
            else:
                in_returns = False
    return node_ids

def migrate_file(filepath):
    with open(filepath, "r") as f:
        lines = f.readlines()
    
    node_ids = get_node_ids(lines)
    file_id = None
    for line in lines:
        if line.strip().startswith("id:"):
            file_id = line.split(":", 1)[1].strip().strip('"').strip("'")
            break
    if file_id in node_ids: node_ids.remove(file_id)
    
    all_vars = base_known_vars + list(node_ids)
    new_lines = []
    changed = False
    in_returns = False
    
    for line in lines:
        original = line
        stripped = line.strip()
        
        # Simple state machine to skip 'returns:' blocks
        if stripped.startswith("returns:"):
            in_returns = True
            new_lines.append(line)
            continue
        
        if in_returns and ":" in line and not line.startswith(" "):
             in_returns = False
        
        if ":" in line and not stripped.startswith("#"):
            left_part, right_part = line.split(":", 1)
            key = left_part.strip()
            raw_val = right_part.strip()
            
            # Skip architectural keys or if we are in a returns block
            if key in id_keys or in_returns:
                if raw_val.startswith("="):
                    val_only = raw_val[1:].strip()
                    new_line = line.replace(raw_val, val_only)
                    new_lines.append(new_line)
                else:
                    new_lines.append(line)
                continue
            
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
            
            if is_intended_expression(inner_val, all_vars) and not inner_val.startswith("="):
                new_val = "=" + inner_val
                escaped = escape_val(new_val)
                new_line = line.replace(raw_val, escaped)
                new_lines.append(new_line)
            else:
                new_lines.append(line)
        else:
            new_lines.append(line)
            
    # Check if any change actually happened
    if len(new_lines) == len(lines):
        for i in range(len(lines)):
            if new_lines[i] != lines[i]:
                changed = True
                break
    else:
        changed = True
        
    if changed:
        with open(filepath, "w") as f:
            f.writelines(new_lines)
        print(f"Migrated: {filepath}")

def main():
    dirs = ["platforms", "crates/ferroflux-testing/fixtures"]
    count = 0
    for d in dirs:
        for root, _, files in os.walk(d):
            for f in files:
                if f.endswith(".yaml") or f.endswith(".yml"):
                    migrate_file(os.path.join(root, f))
                    count += 1
    print(f"Checked {count} files.")

if __name__ == "__main__":
    main()
