# OpenSpec — Quest

This directory holds Quest's [OpenSpec](https://github.com/Fission-AI/OpenSpec)
workspace. From now on, non-trivial development is **spec-driven**: changes are
proposed against the capability specs in [`specs/`](specs/), implemented, then
folded back into those specs.

- **`specs/`** — the living source of truth for *what the game currently does*,
  organized by capability. Each `specs/<capability>/spec.md` is a set of
  `SHALL` requirements with `WHEN`/`THEN` scenarios.
- **`changes/`** — in-flight change proposals (created by `/opsx:propose`).
  Each change carries a `proposal.md`, `design.md`, `tasks.md`, and **delta
  specs** describing how the capability specs should change. When the work
  lands, the deltas are synced/archived into `specs/`.
- **`config.yaml`** — project context and per-artifact rules injected into every
  OpenSpec artifact the AI generates.

## The reverse-engineered baseline (v1)

The specs in `specs/` were **reverse-engineered from the existing
implementation** — they document the game as it is actually coded today, not a
wishlist. Twenty capability specs (208 requirements, 498 scenarios) were each
produced by reading the relevant `src/<module>/` code, its module `CLAUDE.md`,
the root `CLAUDE.md` "Key Constants", and the design notes under `docs/`, then
grounding every number against the source. All twenty pass
`openspec validate --specs --strict` with zero errors.

Because the baseline is grounded in code rather than docs, **when a spec and the
code disagree, that is a bug in one of them** — reconcile it deliberately rather
than silently editing the spec to match a regression.

### Capability index

| Capability | Reqs | What it covers |
|------------|-----:|----------------|
| [`game-loop`](specs/game-loop/spec.md) | 11 | 100ms tick engine, XP-from-kills, autosave, update checks, offline credit, seeded-RNG determinism |
| [`combat`](specs/combat/spec.md) | 11 | Attack cadence, ordered damage/defense pipelines, enemy sourcing, bosses, death handling, weapon gate |
| [`character-progression`](specs/character-progression/spec.md) | 12 | Six attributes, XP→level curve, prestige loop, Prestige Rank bonuses/gates |
| [`zones`](specs/zones/spec.md) | 12 | 50 zones + subzones, boss gating, Expanse, Fracture (12–30) & Loom (31–50) unlock tables |
| [`items`](specs/items/spec.md) | 12 | Seeded generation, ilvl, T0–T9 tier curve, drop/rarity tables, scoring, auto-equip, 7 slots |
| [`enhancement`](specs/enhancement/spec.md) | 8 | Soulforge +0–+10, success/cost tables, failure downgrade, Soul Tithe, discovery |
| [`ascension`](specs/ascension/spec.md) | 9 | Ten tiers, PR cost table, multiplier curve, Deep/pattern gates, combat application |
| [`deep`](specs/deep/spec.md) | 14 | Mercenary expeditions, Layers, guild rank, wall-clock missions, Layer→unlock mapping |
| [`loom`](specs/loom/spec.md) | 13 | Extractor/Shuttle network, Woven Patterns, shuttle caps, WR→PR conversion |
| [`power-cores`](specs/power-cores/spec.md) | 8 | Six passive PR generators, Deep-layer unlocks, fill-cycle accrual, offline catch-up |
| [`fishing`](specs/fishing/spec.md) | 10 | Spot discovery, sessions, 40 ranks, catch rewards, Storm Leviathan hunt |
| [`dungeon`](specs/dungeon/spec.md) | 10 | Discovery, procedural rooms, auto-exploration, boss-key gate, cadence, safe-exit-on-death |
| [`haven`](specs/haven/spec.md) | 10 | Account base, chance discovery, two-branch room tree, explicit-parameter bonus injection |
| [`stormglass`](specs/stormglass/spec.md) | 10 | Soft currency, Storm Sigils, UTC-day sigil rotation, Chrono Surge, Storm Lure |
| [`achievements`](specs/achievements/spec.md) | 10 | 240 account-level achievements, milestones, scoring, titles |
| [`god-items`](specs/god-items/spec.md) | 7 | Asprika / Sleipnir / Megingjord — fixed God-rarity artifacts, passives, auto-equip protection |
| [`time-vault`](specs/time-vault/spec.md) | 9 | git-backed save snapshots on milestones, browse/restore/fork/delete, no auto-prune |
| [`persistence`](specs/persistence/spec.md) | 10 | JSON save files, `QUEST_DIR`, run vs account state, backward-compat contract, silent-wipe hazard |
| [`vessel-act2`](specs/vessel-act2/spec.md) | 12 | Act 2 kill-switch, Zone-50 launch gate (250k PR burn), the Voyage loop |

## Setup

The `/opsx:*` skills (`.claude/skills/openspec-*`) shell out to a bare
`openspec` command, so the [OpenSpec CLI](https://github.com/Fission-AI/OpenSpec)
needs to be on `PATH`. `make setup` installs it (`npm install -g
@fission-ai/openspec@1.5.0` — pinned to the version the skill files were
generated against, per each `SKILL.md`'s `generatedBy` frontmatter). If npm
isn't available, install it manually with the same command, or run
`make openspec-setup` on its own to (re)install just the CLI.

## Working spec-driven from here

Use the OpenSpec skills (installed under `.claude/`):

1. **`/opsx:propose "<idea>"`** — scaffolds a change with proposal, design, and
   tasks, plus delta specs against the affected capability.
2. **`/opsx:apply`** — implement the change's tasks. Verify with the targeted
   commands in the root `CLAUDE.md` "How to Verify Your Change" table, then
   `make check`.
3. **`/opsx:sync`** or **`/opsx:archive`** — fold the change's delta specs back
   into `specs/` so the baseline stays true.

`/opsx:explore` is a read-only thinking-partner mode for investigating before
you propose.

## Known code-vs-docs discrepancies (found during reverse-engineering)

The baseline pass surfaced places where prose docs / comments have drifted from
the code. The specs follow the **code**; these are logged here as cleanup
candidates (fix the doc, or fix the code if the code is the regression):

- **`game-loop` — update-check interval.** The inline comment in the main loop
  says "~30 minutes"; the actual jittered interval is **10–20 min** (uniform
  600–1200s), matching the root `CLAUDE.md` "15min ±5min".
- **`combat` — boss-death retreat.** Docs say "resets to subzone 1"; the code
  retreats to the **highest zone with a defeated boss** (fallback Zone 1) at
  subzone 1. Overworld *mob* death retries the same enemy and only retreats
  after 3 consecutive deaths.
- **`zones` — Fracture unlock gates.** Root `CLAUDE.md` lists only the Deep-layer
  cap; the code enforces a **dual gate** (Deep layer **and** prestige P50–P300
  per band). `src/zones/CLAUDE.md` documents this correctly.
- **`loom` — pattern count & WR→PR rounding.** There are **29** patterns (28
  completable + 1 eternal); the completed/woven counters exclude the eternal
  one. The `131 WR → 302 PR` doc comment is off by one — the code rounds to
  **303** (its unit test asserts 303). A stale comment also claims all six
  extractors unlock immediately; only Ember Spindle starts unlocked.
- **`deep` — stale doc comments.** `mod.rs` says Deep state does **not** survive
  prestige — it does (only a generation counter advances). Gateway expeditions
  are **72h**, not the "24h" a comment claims. Familiarity duration bonuses are
  **-15/-30/-45%**, not the "-10/-20/-30%" in the enum comments.
- **`power-cores` — 48 PR/day is emergent**, not an enforced clamp; it is simply
  the sum of the six rates when all cores are unlocked.
- **`fishing` — Storm Lure is not a guarantee.** `src/fishing/CLAUDE.md` calls it
  a guaranteed-encounter item; the code only **adds a modest odds bonus**.
- **`dungeon` — dead-code multipliers & stale comments.** An unused helper
  returns Elite 1.5 / Boss 2.0, but live enemies use **2.2 / 3.5** table values.
  "Not yet integrated" header comments are stale — discovery is fully wired.
- **`stormglass` — daily rotation** keys off the **UTC calendar date** (midnight
  rollover). A test comment says "11 types"; there are **12**.
- **`challenges` — count is 14, not 15.** The "15" in module docs is the number
  of integration touch-points to add a game, not a 15th minigame.
- **`achievements` — title count is 64 in code** (vs "29 total" in a stale design
  doc), and there are **19** prestige milestones (a comment says 18).
- **`god-items` — no player-facing acquisition** exists yet; the only path is a
  debug/forge action. "Megingjord" is belt lore but occupies the **Ring** slot.
- **`vessel-act2` — voyage duration comment stale.** A comment says the maiden
  crossing takes "~a real month"; the constants work out to **~2 real weeks**.
- **`time-vault` — "remember picks" is a different feature.** That design doc
  describes the prestige equipment-vault selection memory, **not** the git-backed
  Time Vault.
- **`persistence` — Power Cores & god items have no own files.** Power Core state
  lives inside `deep.json`; god items are attributes on equipped items in the
  character save.
