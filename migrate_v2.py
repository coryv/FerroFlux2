import os
import re

dirs_to_scan = ["platforms", "crates/ferroflux-testing/fixtures"]
known_vars = ["inputs", "settings", "platform", "event", "store", "context", "steps", "request", "response", "body", "headers", "query"]
skip_keys = {"id", "name", "type", "description", "category", "platform", "version", 
             "label", "source_id", "source_handle", "target_id", "target_handle", "port", "method", "operation", "level", "operator"}

def is_intended_expression(val: str, key: str) -> bool:
    if key in skip_keys:
        return False

    if "has(" in val or "json(" in val:
        return True
    if any(op in val for op in [" + ", " - ", " * ", " / ", " == ", " != ", " > ", " < ", " ? ", " : ", "&&", "||"]):
        return True
    
    for kv in known_vars:
        if val.startswith(kv + ".") or val == kv:
            return True
            
    # Check if we are referencing a node like `set_a.result`
    # Must have a dot, no spaces, and not be a url or simple text sentence
    if "." in val and " " not in val and not val.startswith("http"):
        # Just an assumption for node references
        return True
        
    return False

def migrate_file(filepath):
    with open(filepath, "r") as f:
        lines = f.readlines()
        
    changed = False
    new_lines = []
    
    for line in lines:
        original = line
        if ":" in line:
            left, right = line.split(":", 1)
            key = left.strip()
            # Handle list items like `- name: foo`
            if key.startswith("- "):
                key = key[2:].strip()
                
            raw_right = right.strip()
            
            # Replace "'INFO'" -> "INFO"
            m = re.match(r'''^(['"])(['"].*?['"])\1(.*)$''', raw_right)
            if m:
                # stripped nested quotes
                inner_quoted = m.group(2)
                raw_right = inner_quoted
                line = left + ": " + raw_right + m.group(3) + "\n"
                
            is_str = False
            inner_str = raw_right
            if raw_right.startswith('"') and raw_right.endswith('"') and len(raw_right) >= 2:
                is_str = True
                inner_str = raw_right[1:-1]
            elif raw_right.startswith("'") and raw_right.endswith("'") and len(raw_right) >= 2:
                is_str = True
                inner_str = raw_right[1:-1]
                
            if is_intended_expression(inner_str, key):
                if not inner_str.startswith('='):
                    if is_str:
                        quote_char = raw_right[0]
                        line = left + f": {quote_char}={inner_str}{quote_char}\n"
                    else:
                        line = left + f": ={inner_str}\n"

        if line != original:
            changed = True
        new_lines.append(line)
        
    if changed:
        with open(filepath, "w") as f:
            f.writelines(new_lines)
        print(f"Migrated: {filepath}")

if __name__ == "__main__":
    count = 0
    for d in dirs_to_scan:
        for root, dirs, files in os.walk(d):
            for f in files:
                if f.endswith(".yaml") or f.endswith(".waml"):
                    path = os.path.join(root, f)
                    migrate_file(path)
                    count += 1
    print(f"Checked {count} files.")
