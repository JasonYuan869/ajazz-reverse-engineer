#!/usr/bin/env bash
# Installs the ajazz time-sync tool as a macOS LaunchAgent.
# It runs when the AK650 keyboard is attached and periodically while connected.
set -euo pipefail

LABEL="com.jasonyuan.ajazz"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

BIN_SRC="$REPO_DIR/target/release/ajazz"
BIN_DIR="$HOME/.local/bin"
BIN_DST="$BIN_DIR/ajazz"

LOG_DIR="$HOME/Library/Logs/ajazz"
AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_DST="$AGENTS_DIR/$LABEL.plist"

# 1. Build if needed and install the binary to a stable location.
if [[ ! -x "$BIN_SRC" ]]; then
    echo "Building release binary..."
    (cd "$REPO_DIR" && cargo build --release)
fi
mkdir -p "$BIN_DIR" "$LOG_DIR" "$AGENTS_DIR"
cp "$BIN_SRC" "$BIN_DST"
echo "Installed binary -> $BIN_DST"

# 2. Render the plist with absolute paths.
sed -e "s|__BIN__|$BIN_DST|g" \
    -e "s|__LOG__|$LOG_DIR|g" \
    "$REPO_DIR/dist/$LABEL.plist" > "$PLIST_DST"
echo "Installed agent  -> $PLIST_DST"

# 3. (Re)load the agent into the current GUI session.
UID_NUM="$(id -u)"
launchctl bootout "gui/$UID_NUM/$LABEL" 2>/dev/null || true
launchctl bootstrap "gui/$UID_NUM" "$PLIST_DST"
launchctl enable "gui/$UID_NUM/$LABEL"
echo "Agent loaded. Logs: $LOG_DIR"
echo "Trigger a run now with: launchctl kickstart -k gui/$UID_NUM/$LABEL"
