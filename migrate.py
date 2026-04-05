import os
import re

dirs_to_scan = ["platforms", "crates/ferroflux-testing/fixtures"]

def migrate_file(filepath):
    with open(filepath, "r") as f:
        lines = f.readlines()
        
    changed = False
    new_lines = []
    
    for line in lines:
        original = line
        
        # 1. Replace nested quotes e.g. level: "'INFO'" -> level: "INFO"
        # Match something like `key: "'value'"`
        line = re.sub(r'''(:.*?)"'(.*?)'"''', r'\1"\2"', line)
        line = re.sub(r'''(:.*?)'"(.*?)"\'''', r'\1"\2"', line)
        
        # 2. Add `=` for complex expressions.
        # Find cases where the right side of the colon is a string that contains CEL indicators like ` + `, `has(`, ` > `, etc.
        if ":" in line:
            parts = line.split(":", 1)
            right = parts[1].strip()
            
            # Check if it is a quoted string
            if right.startswith('"') and right.endswith('"') and len(right) >= 2:
                inner = right[1:-1]
                # If it looks like a complex CEL expression
                indicators = [' + ', 'has(', ' == ', ' > ', ' < ', ' ? ', ' != ']
                if any(ind in inner for ind in indicators) and not inner.startswith('='):
                    new_right = f'"={inner}"'
                    line = parts[0] + ":" + parts[1].replace(right, new_right)

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
