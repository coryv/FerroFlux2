#!/bin/bash
set -e


bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action issues update "Update Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action issues delete "Delete Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action issues transition "Transition Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action comments get "Get Comments"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action issues assign "Assign Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action issues search "Search Issues"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action sprints create "Create Sprint"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action sprints start "Start Sprint"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action sprints get "Get Sprint"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action projects list "List Projects"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action projects create "Create Project"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action users get "Get User"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira action files attach "Attach File"

bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira trigger issues new "New Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira trigger issues updated "Updated Issue"
bash .claude/skills/ferroflux-integration/scripts/scaffold-node.sh jira trigger comments new "New Comment"
