---
name: drive-game
description: Drive the real game in tmux against fixture save states, inspect PNG screenshots of the rendered TUI to verify your own UI changes, and iterate until they render correctly. Use PROACTIVELY after any change to src/ui/, when asked to "screenshot the game", "show me the change", "drive the game", or before opening a PR that touches UI.
---

# Drive the Game

Run the actual game binary in a tmux session against an isolated, pre-built
save state, send real keystrokes, read the real rendered screen, and capture
color PNG screenshots to embed in PRs.

## Building blocks

| Piece | What it does |
|-------|-------------|
| `QUEST_DIR` env var | Redirects all save I/O (`src/core/paths.rs`) to any directory — never touches `~/.quest` |
| `mkstate` binary | Writes character save fixtures for named scenarios into `QUEST_DIR` |
| `--debug` flag | Disables saves, enables debug menu (backtick key) with content jumps: trigger dungeons, fishing, all challenges, Deep, god items, stormglass grants |
| `scripts/screenshot.sh` | tmux pane → ANSI capture → HTML → headless-Chromium PNG (colors + emoji preserved) |

## Fixture scenarios (`mkstate --list`)

- `fresh` — level 1 at Zone 1, nothing discovered
- `midgame` — level 45, P5, Zone 8, rare/epic gear, stormglass
- `endgame` — level 80, P25, Ascension III, Zone 11 (The Expanse), Stormbreaker
- `boss` — midgame with the subzone boss spawning on the first tick
- `custom` — fresh character shaped entirely by override flags

Every scenario accepts override flags, so arbitrary states compose from the
command line without editing code:

```bash
mkstate custom --level 60 --prestige 12 --zone 9 --subzone 3 --gear epic --boss-ready
mkstate midgame --zone 3 --attrs 25 --stormglass 100
```

Flags: `--name --level --prestige --ascension --attrs --zone --subzone
--gear <rarity> --ilvl --stormglass --stormbreaker --boss-ready`
(run `mkstate --help` for details).

Fixtures cover the character save only; account-level systems (Haven, Deep,
Loom, enhancement) start undiscovered — so `--zone` beyond 11 is not honored
without Deep state. Use the debug menu to trigger account systems, or extend
`src/bin/mkstate.rs` with a new scenario/flag when a state can't be reached.

## Recipe

```bash
FX=$(mktemp -d)                                   # isolated save dir
cargo build
QUEST_DIR=$FX ./target/debug/mkstate midgame      # write fixture(s)

tmux kill-session -t quest 2>/dev/null || true
tmux new-session -d -s quest -x 160 -y 45 "QUEST_DIR=$FX ./target/debug/quest --debug"
sleep 3

tmux capture-pane -p -t quest                     # read screen (text)
tmux send-keys -t quest Down                      # navigate; Enter selects
tmux send-keys -t quest Enter                     # dismiss offline-XP modal after load
tmux send-keys -t quest '`'                       # debug menu (--debug only)

scripts/screenshot.sh quest shot.png              # color PNG of the pane

tmux kill-session -t quest                        # cleanup when done
```

Notes:
- After selecting a character, an offline-progress modal usually appears —
  press Enter to dismiss. Loading a fixture may also pop a retroactive
  "Achievements Unlocked" modal; dismiss it before screenshotting.
- Sleep ~0.3s after each keystroke (100ms tick loop); ~2-3s after launch.
- Test responsive layouts by varying pane size: `-x 80 -y 24` vs `-x 200 -y 50`.
- A new tmux session is needed to switch characters (no in-game logout).

## Self-verification loop (the primary use)

You can see the PNGs — Read the screenshot file and inspect the actual
rendering. So after ANY UI change, close the loop yourself instead of
assuming the code is right:

```
1. Make the UI change
2. cargo build && relaunch the tmux session against a fixture
3. Drive to the affected scene
4. scripts/screenshot.sh quest /tmp/check.png, then Read /tmp/check.png
5. LOOK: did the change render as intended? Alignment, truncation,
   colors, overlap with modals/panels, artifacts at panel borders?
6. If wrong: fix, go to 2. If right: capture the final PR screenshot.
```

Iteration tips:
- For fast checks mid-loop, skip the PNG: `tmux capture-pane -p | grep ...`
  asserts on text/layout instantly. Use the PNG when color, emoji width, or
  visual polish matters — and always for the final look.
- Verify at two pane sizes before calling it done (`-x 80 -y 24` smallest
  supported, `-x 200 -y 50` large) — responsive bugs hide in the size you
  didn't try.
- For animated/transient UI (combat effects, ticker, modals), capture 2-3
  frames a second apart and Read each — a single frame can miss flicker or
  a stuck state.
- Don't stop at "it compiles and the screenshot looks plausible" — check the
  specific pixels/cells your change was supposed to affect, and at least one
  neighboring panel for accidental layout shifts.

## Embedding screenshots in a PR

GitHub's API cannot upload comment attachments, so commit the PNGs to the
PR branch and reference them by raw URL:

```bash
mkdir -p docs/screenshots
scripts/screenshot.sh quest docs/screenshots/<feature>-after.png
git add docs/screenshots && git commit -m "docs: add UI screenshots"
```

```markdown
![combat after change](https://raw.githubusercontent.com/stphung/quest/<branch>/docs/screenshots/<feature>-after.png)
```

Capture before/after pairs when changing existing UI: screenshot `main`'s
rendering first (`git stash` or a worktree), then your change, same pane size.

## When to use

- After EVERY change to `src/ui/` — run the self-verification loop before
  considering the change done, then attach at least one screenshot to the PR.
- To reproduce/verify UI bug reports: build the closest fixture, drive to the
  scene, compare against the report.
- Sanity-checking rendering at multiple terminal sizes.
