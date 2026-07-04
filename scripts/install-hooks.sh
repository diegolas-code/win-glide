#!/bin/sh

# Installs the pre-commit hook by copying it to the local .git/hooks directory.
# Works on macOS, Linux, and Windows (via Git Bash / MSYS2).

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GIT_DIR="$(git rev-parse --git-common-dir)"
HOOK_DEST="$GIT_DIR/hooks/pre-commit"

echo "Installing pre-commit hook to $HOOK_DEST..."

# Ensure target hooks directory exists
mkdir -p "$GIT_DIR/hooks"

# Copy the script
cp "$SCRIPT_DIR/pre-commit.sh" "$HOOK_DEST"

# Make executable
chmod +x "$HOOK_DEST"

echo "✅ Pre-commit hook successfully installed!"
