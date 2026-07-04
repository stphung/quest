#!/usr/bin/env bash
# Capture a tmux pane as a PNG screenshot (colors preserved).
#
# Usage: scripts/screenshot.sh <tmux-session> <output.png>
#
# Pipeline: tmux capture-pane -e (ANSI) -> ansi2html.py -> headless Chromium.
# Chromium is resolved from $CHROMIUM, the Playwright install, or PATH.
set -euo pipefail

SESSION="${1:?usage: screenshot.sh <tmux-session> <output.png>}"
OUT="${2:?usage: screenshot.sh <tmux-session> <output.png>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

CHROMIUM="${CHROMIUM:-}"
if [[ -z "$CHROMIUM" ]]; then
    for candidate in /opt/pw-browsers/chromium chromium chromium-browser google-chrome; do
        if command -v "$candidate" >/dev/null 2>&1; then
            CHROMIUM="$candidate"
            break
        fi
    done
fi
if [[ -z "$CHROMIUM" ]]; then
    echo "error: no chromium binary found (set \$CHROMIUM)" >&2
    exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Pane size -> pixel size (14px mono glyphs are ~8.5px wide, 17.5px tall,
# plus the 12px padding on each side of the <pre>). Some glyphs used by the
# game (compass/weather symbols, emoji) fall back to a font whose line height
# exceeds the plain-text 17.5px assumption, so add slack or the bottom rows
# get clipped by the fixed-size screenshot viewport (no scrolling to save us).
read -r COLS ROWS < <(tmux display-message -p -t "$SESSION" '#{pane_width} #{pane_height}')
WIDTH=$(((COLS * 85 + 5) / 10 + 24))
HEIGHT=$(((ROWS * 175 + 5) / 10 + 24 + 100))

tmux capture-pane -e -p -t "$SESSION" > "$WORK/screen.ansi"
python3 "$SCRIPT_DIR/ansi2html.py" "$WORK/screen.ansi" > "$WORK/screen.html"

"$CHROMIUM" --headless --disable-gpu --no-sandbox --hide-scrollbars \
    --force-device-scale-factor=2 \
    --window-size="$WIDTH,$HEIGHT" \
    --screenshot="$WORK/screen.png" \
    "file://$WORK/screen.html" 2>/dev/null

mkdir -p "$(dirname "$OUT")"
mv "$WORK/screen.png" "$OUT"
echo "Wrote $OUT (${COLS}x${ROWS} pane @ ${WIDTH}x${HEIGHT}px)"
