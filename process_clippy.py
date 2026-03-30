import json
import os

def process_clippy(file_path):
    issues_by_package = {}
    
    with open(file_path, 'r') as f:
        for line in f:
            try:
                data = json.loads(line)
                if data.get('reason') == 'compiler-message':
                    package_id = data.get('package_id', 'unknown')
                    manifest_path = data.get('manifest_path', '')
                    
                    if manifest_path:
                        # Extract the directory name containing Cargo.toml
                        package_name = os.path.basename(os.path.dirname(manifest_path))
                    else:
                        # Fallback to package_id extraction
                        if 'path+file://' in package_id:
                            try:
                                package_name = package_id.split('#')[1].split('@')[0]
                                if package_name == "0.1.0": # Version-only fallback
                                     package_name = package_id.split('/')[-1].split('#')[0]
                            except IndexError:
                                package_name = package_id.split('/')[-1].split('#')[0]
                        else:
                            package_name = package_id.split(' ')[0]
                    
                    message = data.get('message', {})
                    level = message.get('level')
                    
                    if level in ['warning', 'error']:
                        rendered = message.get('rendered', '')
                        code = message.get('code', {}).get('code', 'N/A')
                        
                        if package_name not in issues_by_package:
                            issues_by_package[package_name] = []
                        
                        issues_by_package[package_name].append({
                            'level': level,
                            'code': code,
                            'rendered': rendered
                        })
            except json.JSONDecodeError:
                continue

    # Generate Markdown Report
    report = "# Clippy Audit Report\n\n"
    
    if not issues_by_package:
        report += "No issues or warnings found! 🎉\n"
    else:
        for package, issues in issues_by_package.items():
            report += f"## {package}\n\n"
            report += f"Found {len(issues)} issues.\n\n"
            report += "| Level | Code | Description |\n"
            report += "|-------|------|-------------|\n"
            for issue in issues:
                desc = issue['rendered'].split('\n')[0].replace('warning: ', '').replace('error: ', '')
                report += f"| {issue['level'].capitalize()} | `{issue['code']}` | {desc} |\n"
            report += "\n"
            
            # Optionally add detailed rendered output in a collapsible section
            report += "<details>\n<summary>Detailed Output</summary>\n\n"
            report += "```text\n"
            for issue in issues:
                report += issue['rendered'] + "\n"
            report += "```\n"
            report += "</details>\n\n---\n\n"

    return report

if __name__ == "__main__":
    report_md = process_clippy('clippy_output.json')
    with open('clippy_report.md', 'w') as f:
        f.write(report_md)
