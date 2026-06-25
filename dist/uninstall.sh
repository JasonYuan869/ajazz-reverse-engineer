#!/usr/bin/env bash
# Removes the ajazz LaunchAgent.
set -euo pipefail

LABEL="com.jasonyuan.ajazz"
UID_NUM="$(id -u)"
PLIST_DST="$HOME/Library/LaunchAgents/$LABEL.plist"

launchctl bootout "gui/$UID_NUM/$LABEL" 2>/dev/null || true
rm -f "$PLIST_DST"
echo "Removed agent $LABEL. Binary at ~/.local/bin/ajazz left in place."
