# Archived Changes — backported design history

This directory is the **durable design-rationale trail** for Quest: 82 archived
changes reconstructed from the 187 pre-OpenSpec design/plan documents that used
to live under `docs/design/`, `docs/plans/`, `docs/archive/`, and
`docs/superpowers/`. Each change preserves the original design thinking inside
OpenSpec's native structure:

- `proposal.md` — a short backport header (why/what + the capability it relates to)
- `design.md` — the original design doc(s), **verbatim**
- `tasks.md` — the original implementation plan(s), verbatim (or a note if none was recorded)
- `.openspec.yaml` — `backported: true`, dated to the earliest source doc

These are **archived** (not active) changes: they are excluded from `openspec list`
and `openspec validate`. They are history — read them for *how and why* a system
was built. What each system *does now* lives in `openspec/specs/`. **Do not delete
this directory.**

The big systems are grouped so each is one rich change (e.g. `the-deep/` carries
21 source docs, `the-vessel-act2/` 16, `loom/` 15). Distinct decisions mined from
these docs were curated into `docs/decisions.md`.

## Index (by related capability)


### `achievements`

- [`achievement-new-badges/`](achievement-new-badges/)
- [`character-titles/`](character-titles/) (2 docs)
- [`extended-combat-milestones/`](extended-combat-milestones/)
- [`fishing-dungeon-milestones/`](fishing-dungeon-milestones/)
- [`sys-arch-2-analysis/`](sys-arch-2-analysis/)

### `challenges`

- [`chess-minigame/`](chess-minigame/) (2 docs)
- [`flappy-bird/`](flappy-bird/) (2 docs)
- [`go-challenge/`](go-challenge/) (2 docs)
- [`gomoku/`](gomoku/) (2 docs)
- [`minesweeper/`](minesweeper/) (2 docs)
- [`minesweeper-visuals/`](minesweeper-visuals/)
- [`nine-mens-morris/`](nine-mens-morris/) (2 docs)
- [`rune-deciphering/`](rune-deciphering/) (2 docs)
- [`runic-lights/`](runic-lights/) (2 docs)
- [`shard-fusion/`](shard-fusion/) (2 docs)
- [`snake-challenge/`](snake-challenge/)
- [`vault-warden/`](vault-warden/) (2 docs)

### `character-progression`

- [`character-system/`](character-system/) (2 docs)
- [`prestige-multiplier-rebalance/`](prestige-multiplier-rebalance/)
- [`stat-system/`](stat-system/) (2 docs)

### `combat`

- [`3d-ascii-combat/`](3d-ascii-combat/)
- [`combat-balance/`](combat-balance/) (3 docs)
- [`combat-fitness/`](combat-fitness/) (2 docs)
- [`monster-graphics-ux-spec/`](monster-graphics-ux-spec/)

### `deep`

- [`the-deep/`](the-deep/) (21 docs)

### `enhancement`

- [`enhancement-system/`](enhancement-system/) (2 docs)
- [`soulforge-scene-fx/`](soulforge-scene-fx/) (2 docs)

### `fishing`

- [`fishing-scene-atmosphere/`](fishing-scene-atmosphere/) (2 docs)
- [`fishing-system/`](fishing-system/)
- [`halfblock-fishing-boat/`](halfblock-fishing-boat/)
- [`sys-arch-1-analysis/`](sys-arch-1-analysis/)

### `game-loop`

- [`decoupled-timers/`](decoupled-timers/) (3 docs)
- [`improve-time-estimates/`](improve-time-estimates/) (2 docs)

### `god-items`

- [`god-items/`](god-items/) (2 docs)
- [`sleipnir-speed-bonuses/`](sleipnir-speed-bonuses/) (2 docs)

### `haven`

- [`haven/`](haven/) (2 docs)
- [`haven-scene-fx/`](haven-scene-fx/) (2 docs)

### `items`

- [`auto-equip-power/`](auto-equip-power/)
- [`item-power/`](item-power/)
- [`item-system/`](item-system/) (2 docs)
- [`item-tier-system/`](item-tier-system/)
- [`legacy-runes/`](legacy-runes/)

### `loom`

- [`loom/`](loom/) (15 docs)

### `stormglass`

- [`chrono-surge-mission-acceleration/`](chrono-surge-mission-acceleration/) (2 docs)
- [`sigil-matrix/`](sigil-matrix/) (2 docs)
- [`sigil-panel-layout/`](sigil-panel-layout/) (2 docs)
- [`sigil-roll-animation/`](sigil-roll-animation/) (2 docs)
- [`sigils-in-equipment/`](sigils-in-equipment/)
- [`storm-leviathan-timing/`](storm-leviathan-timing/) (2 docs)

### `time-vault`

- [`cloud-sync/`](cloud-sync/) (2 docs)
- [`git-history-persistence/`](git-history-persistence/) (2 docs)
- [`time-vault-ux/`](time-vault-ux/) (2 docs)
- [`vault-remember-picks/`](vault-remember-picks/)

### `vessel-act2`

- [`the-vessel-act2/`](the-vessel-act2/) (16 docs)

### `zones`

- [`fracture-zone-backgrounds/`](fracture-zone-backgrounds/) (2 docs)
- [`postgame-zones/`](postgame-zones/) (2 docs)
- [`six-fracture-chapters/`](six-fracture-chapters/) (2 docs)
- [`zone-background-scenes/`](zone-background-scenes/) (2 docs)
- [`zone-progression/`](zone-progression/)
- [`zone-system/`](zone-system/) (2 docs)

### Tooling / UI / architecture (no single capability)

- [`auto-update/`](auto-update/) (2 docs)
- [`balance-simulator/`](balance-simulator/) (2 docs)
- [`balance-tuning/`](balance-tuning/) (2 docs)
- [`credits-screen/`](credits-screen/)
- [`debug-menu/`](debug-menu/)
- [`debug-menu-character-tab/`](debug-menu-character-tab/) (2 docs)
- [`debug-menu-shortening-mockups/`](debug-menu-shortening-mockups/)
- [`loot-ticker/`](loot-ticker/) (2 docs)
- [`meta-audit-skill/`](meta-audit-skill/) (2 docs)
- [`pat-update-flow/`](pat-update-flow/) (2 docs)
- [`perf-audit-skill/`](perf-audit-skill/) (2 docs)
- [`pr-checks/`](pr-checks/)
- [`responsive-ui/`](responsive-ui/) (5 docs)
- [`scaffold-wiring/`](scaffold-wiring/) (2 docs)
- [`structural-overhaul/`](structural-overhaul/) (2 docs)
- [`terminal-idle-rpg/`](terminal-idle-rpg/)
- [`ticker-m-s-only/`](ticker-m-s-only/)
- [`ticker-slo-speed/`](ticker-slo-speed/)
- [`title-screen-stats/`](title-screen-stats/)
- [`unified-right-panel/`](unified-right-panel/)
- [`unified-status-strip/`](unified-status-strip/) (2 docs)
- [`wiki-link-feature/`](wiki-link-feature/) (2 docs)
