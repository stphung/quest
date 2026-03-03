#!/bin/bash
# Worktree-aware cargo fmt hook for Claude Code.
# Extracts file_path from TOOL_INPUT and runs cargo fmt from the correct directory.

FILE_PATH=$(echo "$TOOL_INPUT" | grep -o '"file_path":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$FILE_PATH" ]; then
    cargo fmt 2>/dev/null
    exit 0
fi

# Walk up from the edited file to find the nearest Cargo.toml
DIR=$(dirname "$FILE_PATH")
while [ "$DIR" != "/" ] && [ "$DIR" != "." ]; do
    if [ -f "$DIR/Cargo.toml" ]; then
        cargo fmt --manifest-path "$DIR/Cargo.toml" 2>/dev/null
        exit 0
    fi
    DIR=$(dirname "$DIR")
done

cargo fmt 2>/dev/null
