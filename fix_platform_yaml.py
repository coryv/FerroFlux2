#!/usr/bin/env python3
"""
Fix Gemini's YAML migration bugs in platform definitions.
Uses precise line-by-line transforms to avoid unintended matches.
"""
import os
import yaml

def fix_lines(lines):
    """Fix a list of lines from a platform YAML file."""
    result = []
    in_execution = False
    in_interface = False
    depth_stack = []  # track indent depth changes

    for line in lines:
        stripped = line.rstrip('\n')
        lstripped = stripped.lstrip()
        indent = len(stripped) - len(lstripped)

        # Bug B1: Remove = from execution step IDs
        # Matches: "  - id: =word" (exactly 2-space indent)
        if indent == 2 and lstripped.startswith('- id: =') and ' ' not in lstripped[7:].split('\n')[0]:
            word = lstripped[7:]
            if word.isidentifier() or all(c.isalnum() or c == '_' for c in word):
                stripped = '  - id: ' + word
                result.append(stripped + '\n')
                continue

        # Bug B2: Remove = from interface port/settings schema names
        # Matches: "    - name: =word" (exactly 4-space indent)
        if indent == 4 and lstripped.startswith('- name: ='):
            word = lstripped[9:]
            # Only fix single-word names (not expressions)
            if word.isidentifier():
                stripped = '    - name: ' + word
                result.append(stripped + '\n')
                continue

        # Bug B3: Remove = from tool names
        # Matches: "    tool: =word" (exactly 4-space indent) OR "      - tool: =word" (6-space list item in routing)
        # Tool names can contain dots, underscores, alphanumeric (e.g. core.utils.json, http_client, emit)
        def is_valid_tool_name(s):
            return bool(s) and all(c.isalnum() or c in '._' for c in s) and not s.startswith('.')

        if indent == 4 and lstripped.startswith('tool: ='):
            word = lstripped[7:]
            if is_valid_tool_name(word):
                stripped = '    tool: ' + word
                result.append(stripped + '\n')
                continue

        # Also fix "- tool: =word" inside routing cases (6-space indent list item)
        if indent == 6 and lstripped.startswith('- tool: ='):
            word = lstripped[9:]
            if is_valid_tool_name(word):
                stripped = '      - tool: ' + word
                result.append(stripped + '\n')
                continue

        result.append(line)

    return result

def fix_http_client_params(content):
    """
    Fix the headers and body params in http_client execution steps to use has() guards.
    These fail with 'No such key' when the user doesn't provide optional settings.
    """
    # Fix: headers: =self.settings.headers -> with has() guard
    old = '      headers: =self.settings.headers\n'
    new = '      headers: "=has(self.settings.headers) ? self.settings.headers : null"\n'
    content = content.replace(old, new)

    # Fix: body: =self.settings.body -> with has() guard
    old = '      body: =self.settings.body\n'
    new = '      body: "=has(self.settings.body) ? self.settings.body : null"\n'
    content = content.replace(old, new)

    return content

def main():
    changed_count = 0
    error_count = 0

    for root, dirs, files in os.walk('platforms'):
        for f in files:
            if not f.endswith('.yaml'):
                continue
            path = os.path.join(root, f)

            with open(path, 'r') as fh:
                original = fh.read()

            lines = original.splitlines(keepends=True)
            fixed_lines = fix_lines(lines)
            fixed = ''.join(fixed_lines)

            # Apply http_client has() guard fixes
            fixed = fix_http_client_params(fixed)

            if fixed == original:
                continue

            # Validate the fixed content parses as YAML
            try:
                yaml.safe_load(fixed)
            except yaml.YAMLError as e:
                print(f'  SKIP (would break YAML): {path}: {e}')
                error_count += 1
                continue

            with open(path, 'w') as fh:
                fh.write(fixed)
            changed_count += 1

    print(f'Summary: {changed_count} files fixed, {error_count} skipped')

if __name__ == '__main__':
    main()
