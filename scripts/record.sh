#!/usr/bin/env bash
# Record a tmux pane as a video (colors preserved) by sampling frames at a
# fixed interval and encoding them with ffmpeg. Useful for reviewing
# animations/transitions that a single screenshot can't show.
#
# Usage: scripts/record.sh <tmux-session> <output.webm|mp4|gif> [duration_s] [fps]
#
# Pipeline: tmux capture-pane -e (ANSI, sampled every 1/fps seconds) ->
# ansi2html.py -> headless Chromium screenshot per frame -> ffmpeg encode.
#
# Send keystrokes with `tmux send-keys` in another terminal (or a backgrounded
# subshell) while this script is sampling, to capture an interaction.
set -euo pipefail

SESSION="${1:?usage: record.sh <tmux-session> <output.webm|mp4|gif> [duration_s] [fps]}"
OUT="${2:?usage: record.sh <tmux-session> <output.webm|mp4|gif> [duration_s] [fps]}"
DURATION="${3:-5}"
FPS="${4:-5}"
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

FFMPEG="${FFMPEG:-}"
if [[ -z "$FFMPEG" ]]; then
    for candidate in ffmpeg /opt/pw-browsers/ffmpeg-*/ffmpeg-linux; do
        if command -v "$candidate" >/dev/null 2>&1; then
            FFMPEG="$candidate"
            break
        fi
    done
fi
if [[ -z "$FFMPEG" ]]; then
    echo "error: no ffmpeg binary found (set \$FFMPEG)" >&2
    exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Some glyphs used by the game (compass/weather symbols, emoji) fall back to
# a font whose line height exceeds the plain-text 17.5px assumption, so add
# slack or the bottom rows get clipped by the fixed-size screenshot viewport
# (no scrolling to save us) -- see the matching note in screenshot.sh.
read -r COLS ROWS < <(tmux display-message -p -t "$SESSION" '#{pane_width} #{pane_height}')
WIDTH=$(((COLS * 85 + 5) / 10 + 24))
HEIGHT=$(((ROWS * 175 + 5) / 10 + 24 + 100))
# ffmpeg's video encoders require even dimensions.
WIDTH=$((WIDTH + WIDTH % 2))
HEIGHT=$((HEIGHT + HEIGHT % 2))

FRAME_COUNT=$((DURATION * FPS))
INTERVAL=$(awk -v fps="$FPS" 'BEGIN { printf "%.3f", 1 / fps }')

echo "Sampling $FRAME_COUNT frames at ${FPS}fps (${DURATION}s) from '$SESSION'..." >&2
for ((i = 0; i < FRAME_COUNT; i++)); do
    tmux capture-pane -e -p -t "$SESSION" > "$WORK/frame_$(printf '%04d' "$i").ansi"
    sleep "$INTERVAL"
done

echo "Rendering frames to PNG..." >&2
for ((i = 0; i < FRAME_COUNT; i++)); do
    N="$(printf '%04d' "$i")"
    python3 "$SCRIPT_DIR/ansi2html.py" "$WORK/frame_$N.ansi" > "$WORK/frame_$N.html"
    "$CHROMIUM" --headless --disable-gpu --no-sandbox --hide-scrollbars \
        --force-device-scale-factor=2 \
        --window-size="$WIDTH,$HEIGHT" \
        --screenshot="$WORK/frame_$N.png" \
        "file://$WORK/frame_$N.html" 2>/dev/null
done

echo "Encoding video..." >&2
mkdir -p "$(dirname "$OUT")"
"$FFMPEG" -y -framerate "$FPS" -i "$WORK/frame_%04d.png" \
    -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" \
    -pix_fmt yuv420p "$OUT" 2>&1 | tail -5

echo "Wrote $OUT (${COLS}x${ROWS} pane, ${DURATION}s @ ${FPS}fps)"
