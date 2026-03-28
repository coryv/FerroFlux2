#!/bin/bash
set -e
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive action files get-metadata "Get File Metadata"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive action files list "List Files"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive action files search "Search Files"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive action permissions create "Share File"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive trigger files new "New File"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh google-drive trigger files updated "File Updated"

bash .claude/skills/ferroflux-integration/scripts/scaffold-platform.sh anthropic "Anthropic Claude" "https://api.anthropic.com/v1" "custom:x-api-key" "AI/ML"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh anthropic action messages create "Create Message"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh anthropic action messages stream "Stream Message"
