# Design Decisions Log

Key decisions made during development, with rationale. Organized by system.

## Prestige Multiplier Formula

**Explored three formulas:**

| Formula | P10 | P20 | P30 | Issue |
|---------|-----|-----|-----|-------|
| `1.5^rank` | 57.7x | 3,325x | 191K× | Hyper-exponential, trivializes everything |
| `1.2^rank` | 6.2x | 38.3x | 237x | Still too fast — later cycles become shorter than earlier ones |
| `1+0.5*rank^0.7` | 3.5x | 5.1x | 6.4x | **Chosen.** Diminishing returns, asymptotes ~6-7x |

**Decision**: Sub-linear formula `1 + 0.5 * rank^0.7`. This preserves the "wall → reset → power fantasy" loop at every stage. Early prestiges feel impactful (+50% at P1), while late-game requires genuine time investment.

## Zone Count: 10 vs 20

**Original design**: 20 zones with an "Era 2: Planar Journey" (Zones 11-20) requiring weapon forging + multi-phase bosses at each zone gate.

**Implemented**: 10 zones + 1 infinite post-game zone (The Expanse).

**Rationale**: 20 zones would require ~10x more enemy types, sprites, boss mechanics, and weapon definitions. The 10-zone structure provides a complete arc (Nature → Cosmic) with the Stormbreaker quest chain as a satisfying endgame gate. The Expanse provides infinite replay without needing 10 more authored zones.

## Zone Progression Design: Competing Proposals

Two zone designs were written before implementation:

| Aspect | 8-Zone Design | 20-Zone Design |
|--------|---------------|----------------|
| Zones | 8, level-gated | 20, prestige-gated |
| Prestige mult | Diminishing returns (asymptote ~6x) | `1.2^rank` |
| Endgame | Zone 8 at P15 | Weapon forging chain per zone |
| Level cap | `20 + prestige * 15` | None |

**Implemented**: A hybrid — 10 prestige-gated zones (from the 20-zone design's structure) with no level cap, plus the Stormbreaker endgame gate as a single weapon quest instead of per-zone forging.

## Stormbreaker: Drop vs Forge

**Original design**: Stormbreaker as a random Legendary item drop (pure RNG).

**Implemented**: Multi-system quest chain (fishing → Haven → forge). This was chosen because:
- A random drop for a progression gate feels bad (no agency)
- The quest chain ties together three systems (fishing, Haven, prestige spending)
- It creates a clear endgame goal that players can plan toward
- The ~month timeline for Storm Leviathan fishing matches the intended pace

## Haven Bonus Types: Design vs Implementation

Several Haven bonuses changed from design to implementation:

| Room | Designed As | Implemented As | Reason |
|------|------------|----------------|--------|
| War Room | Attack interval reduction | Double Strike chance | More exciting, avoids changing tick timing |
| Fishing Dock | Fishing rank XP boost | Double Fish chance + Max Rank increase | Extends fishing system depth (ranks 31-40) |
| Vault | 1/2/3 items | 1/3/5 items | Higher ceiling for invested players |
| Haven currency | Prestige ranks + Fishing ranks | Prestige ranks only | Simplified economy, fishing ranks used for Dock T4 |

## Equipment Reset on Prestige

**Decision**: Equipment is completely wiped on prestige (all 7 slots cleared).

**Rationale**: Prestige should feel like a meaningful reset. Keeping equipment would trivialize early zones on each cycle. The Vault Haven room provides a controlled way to preserve 1-5 items for invested players, making it an earned perk rather than a default.

## Save Format: Binary vs JSON

**Original design**: Binary format with bincode for speed.

**Implemented**: Plain JSON with serde, no checksum.

**Rationale**: JSON is human-readable, debuggable, and trivially compatible with serde. Save files are small (<10KB), so binary encoding offers no meaningful performance benefit. The original design called for SHA256 checksums but these were never implemented — serde's structural validation on load is sufficient to catch corruption.

## Challenge Discovery Weights

Not all challenges are equally discoverable. This reflects the original 6-game roster's
rationale at ship time; 8 more minigames have since been added and weights have been
retuned. For the current weight table, see `src/challenges/CLAUDE.md` (Discovery
Weights) or `CHALLENGE_TABLE` in `src/challenges/menu.rs`.

| Challenge | Weight | Rationale |
|-----------|--------|-----------|
| Minesweeper (30) | Most common | Most accessible, familiar mechanics |
| Rune (25) | Common | Simple to learn, quick games |
| Gomoku (20) | Medium | Moderate complexity |
| Morris (15) | Less common | Niche game, less recognizable |
| Chess (10) | Rare | Most complex, intimidating for casual players |
| Go (10) | Rare | Steepest learning curve |

## AI Algorithms Per Game

| Game | Algorithm | Why |
|------|-----------|-----|
| Chess | Minimax (via chess-engine crate) | Established, crate handles move validation |
| Morris | Minimax + alpha-beta | Low branching factor, minimax works well |
| Gomoku | Minimax + alpha-beta | Line-based evaluation is natural for minimax |
| Go | MCTS | Branching factor ~80 makes minimax impractical; no reliable eval function for Go |
| Minesweeper | N/A (puzzle) | Single-player, no AI opponent |
| Rune | N/A (puzzle) | Single-player deduction |

## Fishing: 40 Ranks vs 30

**Original design**: 30 ranks across 6 tiers.

**Implemented**: 40 ranks across 8 tiers, with ranks 31-40 locked behind FishingDock T4.

**Rationale**: The Storm Leviathan quest requires Rank 40 as a prerequisite. This gates the Stormbreaker behind significant fishing investment and makes the FishingDock T4 upgrade meaningful. The extended ranks (Mythic/Transcendent tiers) also provide a long-term goal for completionists.

## Offline XP: Kill Simulation vs Passive Ticks

**Decision**: Offline progression simulates kills rather than accumulating passive tick XP.

**Rationale**: Kill-based XP is the primary source in active play. Simulating kills (at 25% efficiency) keeps offline and online progression on the same curve, just slower. Pure tick-based offline XP would be disconnected from actual gameplay pacing.

## Haven Discovery: Separate RNG

**Decision**: Haven discovery uses its own RNG roll per tick, independent from challenge discovery.

**Rationale**: Haven requires P10+ (much later than challenges at P1+). Sharing the RNG roll with challenges would mean Haven competes with challenge discovery, potentially delaying one or the other. Separate rolls mean a P10+ player can discover both Haven and challenges independently.

## game_tick() Extraction to core/tick.rs

**Decision**: Extract the per-tick orchestration function from main.rs into `src/core/tick.rs`, returning a `TickResult` struct with `Vec<TickEvent>` instead of mutating UI state directly.

**Rationale**: The game loop was tightly coupled to the terminal UI — game logic called `add_log_entry()` and created `VisualEffect` objects directly. Extracting `game_tick()` into a pure-logic module enables:
- Headless simulation (the `simulator` binary reuses the exact same function)
- Testable game logic without terminal dependencies
- Clear separation: tick.rs has zero `ui::` imports

**Key choices**:
- **Generic `<R: Rng>`** instead of `&mut dyn Rng` because `rand::Rng` is not dyn-compatible. Production passes `&mut thread_rng()`, tests use seeded `ChaCha8Rng`.
- **Pre-formatted messages** in TickEvent variants (with unicode escapes) rather than raw data. The presentation layer uses them directly.
- **`achievements_changed` / `haven_changed` flags** signal when IO (disk save) is needed, keeping file I/O in main.rs.
- **Fishing early return**: fishing and combat are mutually exclusive within a tick (stage 5 returns early, skipping stages 6-7).

## tick_events.rs Extraction from main.rs

**Decision**: Extract the TickEvent-to-UI mapping code from main.rs into `src/tick_events.rs`.

**Rationale**: After `game_tick()` returns a `TickResult`, main.rs still needed ~130 lines of match arms to convert `TickEvent` variants into combat log entries and visual effects. This bridge code is binary-only (not part of `lib.rs`) because it imports UI types (`VisualEffect`, `EffectType`). Extracting it into its own module keeps main.rs focused on the game loop, input handling, and screen management.

## offline.rs Extraction from game_logic.rs

**Decision**: Extract offline progression functions (`calculate_offline_xp`, `process_offline_progression`, `OfflineReport`) from `game_logic.rs` into `src/core/offline.rs`.

**Rationale**: Offline progression is a self-contained subsystem with its own types and test suite. Extracting it reduces `game_logic.rs` size and makes the offline XP formula easier to find, test, and modify independently. Re-exports in `game_logic.rs` maintain backwards compatibility.

## Challenge Standardization: Forfeit Pattern and AI Naming

**Decision**: All interactive minigames use the same forfeit pattern (first Esc sets `forfeit_pending`, second Esc confirms, any other key cancels) and all AI games use `process_ai_thinking()` as the function name.

**Rationale**: Before standardization, each minigame had slightly different forfeit handling and AI function names (e.g., `process_go_ai`, `tick_chess`). Consistent patterns make it easier to add new minigames — the `challenges/CLAUDE.md` checklist documents the exact template.

## Debug Mode Autosave Fix: Always Sync last_save_time

**Decision**: In the autosave timer, always sync `state.last_save_time = Utc::now().timestamp()` regardless of debug mode. Only skip the file I/O (`save_character()`, `save_haven()`) in debug mode.

**Rationale**: Previously, debug mode skipped the entire autosave block, including the `last_save_time` sync. This caused the suspension detection system (which checks wall-clock time vs `last_save_time`) to false-trigger after ~60 seconds of debug play, showing an incorrect offline XP report. The fix separates the in-memory timestamp sync from the file I/O skip.

## Headless Game Simulator for Balance Testing

**Decision**: Add a `src/bin/simulator/` binary that runs the game tick loop headlessly, collecting metrics for game balance analysis.

**Rationale**: Balance testing previously required playing the game manually or writing one-off test harnesses. The simulator reuses the exact same `game_tick()` function, ensuring perfect fidelity with the real game. It supports:
- Configurable tick count, RNG seed, starting prestige
- Multi-run aggregation with min/avg/max statistics
- CSV time-series export for graphing progression curves
- Verbose per-tick event logging for debugging

This enables systematic balance validation: "does a P0 character reach Zone 2 in 1 hour?" or "what's the item drop distribution over 10,000 ticks across 100 seeds?"

## Soulforge Enhancement System

**Decision**: Add an account-level equipment enhancement system (Soulforge) that enhances slots (not individual items), gated behind P15+, with independent discovery RNG.

**Key choices**:
- **Slot-based enhancement** instead of item-based: Enhancement levels persist across prestige resets and item swaps. Since items are lost on prestige (except Vault), item-based enhancement would feel punishing. Slot-based enhancement means the investment carries forward permanently.
- **Independent discovery at P15+**: The Soulforge uses its own RNG roll per tick (same formula as Haven: `0.000014 + (rank - 15) * 0.000007`). Gating at P15 ensures players have established their Haven before discovering enhancement, avoiding early resource competition.
- **Prestige ranks as currency**: Same currency as Haven, creating meaningful resource allocation decisions between Haven upgrades and equipment enhancement.
- **Escalating risk/reward**: Levels +1-4 are safe (100% success), while +5-10 have decreasing success rates and failure penalties (-1 or -2 levels). This creates a natural progression from safe investment to high-risk gambling, with the +10 level (10% success, -2 on failure) being a prestige sink for endgame players.
- **Account-level persistence**: Stored in `~/.quest/enhancement.json` alongside Haven and achievements.

## Containment Breach (JezzBall) Challenge

**Decision**: Add a JezzBall-inspired real-time action minigame where players split an arena with growing walls while avoiding bouncing hazard orbs.

**Key choices**:
- **Wall-growing mechanic**: Walls expand from a pivot point in both directions along the chosen axis, creating a tense timing element. Players must judge when it's safe to place walls based on ball trajectories.
- **3 lives system**: Rather than instant game-over on a single collision, players get 3 lives. When a ball hits a growing wall, the wall is destroyed, a life is consumed, and the game resets to a waiting state (preserving captured territory and ball positions). This matches Skyward Gauntlet's 3-life system and reduces frustration from single unlucky collisions.
- **Area capture via flood fill**: When a wall completes (reaches existing barriers on both ends), regions not containing balls are automatically captured. This avoids complex territory calculation and provides satisfying visual feedback.
- **Difficulty scaling via ball count and target**: Novice (2 balls, 60% target) to Master (5 balls, 84% target), with increasing ball speed and faster wall growth intervals.

## Challenge Discovery Weight Rebalance

**Decision**: Rebalance challenge discovery weights from the original 6-game distribution to accommodate 10 games (adding Snake, Flappy Bird, JezzBall, and Sigil Surge).

**Original distribution** (6 games, total weight 110):
- Minesweeper: 30, Rune: 25, Gomoku: 20, Morris: 15, Chess: 10, Go: 10

**New distribution** (10 games, total weight 180):
- Rune: 30, Minesweeper: 28, Snake: 22, Flappy Bird: 20, Sigil Surge: 20, JezzBall: 18, Gomoku: 15, Morris: 12, Chess: 8, Go: 7

**Rationale**: The rebalance follows the principle that quick, accessible games should appear more frequently. Action games (Snake, Flappy Bird, JezzBall, Sigil Surge) are placed in the middle tier since they offer moderate play sessions. Strategy games (Chess, Go) were reduced slightly to make room for the new entries while maintaining their "rare discovery" feel. Rune was promoted to the top weight as the fastest challenge (~2 minutes).

## Phase 2 Large Module Refactoring (PRs #288, #291, #292)

**Decision**: Extract ~33 submodules from 7 large logic files across core, combat, character, fishing, haven, achievements, main.rs, and input.rs.

**What changed**:
- `core/tick.rs` split into `tick.rs` (orchestrator), `tick_types.rs` (TickEvent/TickResult), `tick_stages.rs` (stages 4-6 + helpers), `xp.rs` (XP calculation), `discoveries.rs` (discovery rolls)
- `core/game_logic.rs` thinned to a re-export wrapper; logic moved to `enemy_spawning.rs`, `xp.rs`, `recent_drops.rs`, `ticker.rs`
- `combat/logic.rs` split into `logic.rs` (orchestrator), `player_attack.rs`, `enemy_attack.rs`, `damage.rs`, `events.rs`, `regen.rs`
- `character/manager.rs` split into `manager.rs`, `persistence.rs`, `name_validation.rs`
- `fishing/logic.rs` split into `logic.rs`, `discovery.rs`, `drops.rs`, `rank.rs`
- `haven/types.rs` split into `types.rs`, `room_defs.rs`, `bonus.rs`
- `achievements/data.rs` split into `data.rs`, `handlers.rs`, `milestones.rs`
- `main.rs` helpers extracted into `main_helpers/` directory (achievements, character_screens, input_routing, offline, overlay, persistence, scene, update)
- `input.rs` promoted to `input/` directory with submodules (haven_input, minigame_input, prestige_input, soulforge_input, types)

**Why**: The largest files exceeded 1000 LOC (main.rs 1548, character/manager.rs 1396, dungeon/logic.rs 1217, fishing/logic.rs 1047, core/game_logic.rs 1012), making navigation and maintenance difficult. The docs/plans/2026-02-18-large-module-refactoring-design.md design document identified the candidates.

**Pattern**: Move focused logic into sibling files within the same module, keep the original file as a thin orchestrator, and re-export all public symbols from `mod.rs` for backward compatibility. No public API changes.

**Result**: All tests pass. No changes to module public APIs. Callers unaffected.

## Phase 3 Large Module Refactoring (PR #294)

**Decision**: Extract ~23 additional submodules from 10 files across character, combat, zones, dungeon, achievements, and UI modules.

**What changed**:
- `character/prestige.rs` split into `prestige.rs`, `combat_bonuses.rs`, `multipliers.rs`, `prestige_actions.rs`, `tiers.rs`
- `character/derived_stats.rs` extracted `calculation.rs` for the stats calculation engine
- `character/input.rs` split into `input.rs` (router), `creation.rs`, `delete.rs`, `rename.rs`, `select.rs`
- `combat/logic.rs` extracted `orchestration.rs` (update_combat), `attacks.rs` (intervals), `enemy_generation.rs` (zone/dungeon generators)
- `zones/progression.rs` extracted `advancement.rs`, `boss_defeat.rs`, `gates.rs`
- `dungeon/logic.rs` extracted `pathfinding.rs`, `rewards.rs`
- `achievements/types.rs` extracted `modal.rs`, `notifications.rs`, `stats.rs`, `unlock.rs`
- UI modules extracted rendering submodules: `stats_attributes.rs`, `stats_equipment.rs`, `stats_prestige.rs`, `haven_details.rs`, `haven_tree.rs`, `achievement_details.rs`, `achievement_list.rs`, `achievement_tabs.rs`, `soulforge_effects.rs`, `soulforge_slots.rs`, `enemy_sprite_data.rs`

**Why**: Continuation of Phase 2 refactoring. Remaining large files still exceeded maintainability thresholds. Same pattern: extract focused logic into sibling files, keep original as thin orchestrator, re-export all public symbols.

**Result**: All tests pass. No changes to module public APIs. Callers unaffected.

## PR #300: Challenge Macro and Final Submodule Extractions

**Decision**: Introduce `impl_apply_game_result!` macro and extract remaining AI submodules.

**What changed**:
- Added `impl_apply_game_result!` macro in `src/challenges/mod.rs` to standardize reward application across all challenge minigames (12 at the time, now 14)
- Extracted `morris/ai.rs` and `gomoku/ai.rs` as separate AI submodules
- Extracted `combat/enemy_generation.rs` for zone/dungeon enemy generators

**Why**: The reward application code was duplicated across all challenge modules with minor variations. The macro eliminates this duplication and ensures consistent behavior. AI extraction follows the same submodule pattern established in Phases 2 and 3.

## Stormglass Currency System

**Decision**: Add Stormglass as a character-level currency earned passively through gameplay (item salvage, dungeon caches, soulforge consolation, challenge rewards) and spent at the Stormglass Exchange overlay for four options: Invoke Challenge, Chrono Surge, Storm Sigils, and Storm Lure.

**Rationale**: Stormglass provides a secondary progression sink for endgame players (P15+) and gives value to non-equipped item drops via auto-salvage. Storm Sigils offer permanent percentage-based bonuses that scale with investment, while Invoke Challenge and Chrono Surge give players agency over discovery pacing. The Storm Lure ties Stormglass into the Stormbreaker quest chain.

## Time Vault: Git-Based Save Versioning

**Decision**: Implement a git-based save versioning system (Time Vault) that creates commits on meaningful game events and allows players to browse, restore, and fork save branches. Includes optional GitHub cloud sync for cross-device play.

**Rationale**: Players invest significant time in idle RPG progression. Git provides built-in branching, history, and diffing without inventing a custom versioning system. Cloud sync via GitHub PAT enables cross-device play while keeping the system simple (no custom server infrastructure).

## Combat Retreat: Death Loop Prevention

**Decision**: Add a combat retreat system (`DEATH_LOOP_THRESHOLD = 3`, `MOB_FIGHT_TIMEOUT_SECONDS = 30.0`) that automatically retreats the player to a safer zone when they die repeatedly or stall against a mob.

**Rationale**: Without intervention, a player who enters a zone too early can get stuck in a death loop — dying instantly, respawning, and dying again. The retreat system detects consecutive deaths and mob fight timeouts, then moves the player back to a zone they can handle, preserving the gameplay loop.

## Frontier Backoff: Macro Death Loop Prevention

**Decision**: When a *death-triggered* combat retreat fires, remember the zone that caused it (`ZoneProgression::record_death_retreat()`). Boss-defeat advancement then cycles the safe zone instead of auto-advancing back into the recorded zone, consuming a cooldown that grows with repeated retreats (capped at `FRONTIER_BACKOFF_MAX_CYCLES = 8`). Defeating any boss inside the recorded zone — or prestiging — clears the memory.

**Rationale**: The retreat system alone creates a macro loop at zone frontiers (#576): retreat sends the player to the zone they just cleared, they re-defeat its boss, auto-advance back into the killer zone, die three more times, and repeat indefinitely. Backoff converts the tight bounce into progressively longer farming stints in the safe zone, so the player keeps earning XP/levels until they can actually survive the frontier. Stalemate (mob timeout) retreats deliberately do *not* record backoff — the player survives those, retrying is cheap, and forcing extra safe-zone cycles for stalemates measurably starves leveling at the Loom frontier.

## Ascension System: Per-Character Prestige-Rank Multiplier

**Decision**: Add a per-character combat power multiplier (Ascension I-VI+) purchased with prestige ranks and gated by Deep layer milestones. Ascension level survives prestige.

**Rationale**: Fracture zones (12-30) scale enemy stats at 1.6x per zone, creating a steep power wall. Ascension provides a structured catch-up mechanism without trivializing early zones: players must unlock Deep layers before buying Ascension levels, ensuring progression feels earned. Surviving prestige means each purchase is permanent infrastructure, not a short-term boost.

**Key choices**:
- **Multiplier formula**: `2^level` for I-VI (doubling per level, 2x to 64x), then `64 * 1.5^(level-6)` for VII+ (diminishing 1.5x returns). Provides strong early scaling with smoother late scaling.
- **Deep layer gates**: [3, 7, 12, 18, 25, 30] — each Ascension level aligns with a Deep breakthrough, tying two systems together.
- **PR cost table**: [35, 65, 120, 200, 325, 500] for I-VI (total 1,245 PR). Costs are steep enough that purchasing all 6 requires dedicated prestige investment over many cycles.
- **Multiplier scope**: Applies to damage, defense, and max HP equally, keeping the combat feel balanced rather than just inflating one stat.

## Power Rating: Combat Strength Scalar

**Decision**: Add a single `power_rating` metric computed as `sqrt(effective_DPS × effective_HP)` (geometric mean of offense and defense).

**Rationale**: With many bonus sources (prestige, Haven, god items, sigils, ascension, enhancement), a single comparable number is valuable for both player guidance and developer balance analysis. The geometric mean rewards balanced builds: a character with 1000 DPS and 1000 effective HP rates better than one with 2000 DPS and 100 HP (geometric mean penalizes imbalance). The square root normalizes large numbers for display.

**Key choices**:
- **Formula**: `sqrt(effective_DPS × effective_HP)` — accounts for the full damage pipeline (crit, double strike, attack speed) and defense reduction multiplier.
- **Caching**: Stored as `cached_power_rating: f64` on `GameState`, recomputed each tick alongside other stat recalculation.
- **Display**: Shown in the stats panel header (same row as player name/level), giving it permanent visibility.

## Fracture Zones 12-30: Deep-Unlocked Chapters

**Decision**: Add 19 fracture zones (Z12-30) across 6 chapters, each chapter unlocked by a Deep layer breakthrough. Enemy stats scale at 1.6x per zone from Zone 11 base.

**Rationale**: The Expanse (Zone 11) provides infinite content but at a fixed difficulty. Fracture zones create a long-term power progression goal that requires mastering The Deep and Ascension — two of the deepest endgame systems. Tying zone unlocks to Deep breakthroughs means advancing in one system always rewards the other.

**Key choices**:
- **1.6x stat multiplier per zone**: Steep enough to require real power investment (specifically Ascension), but not so steep that any single zone feels impossible after buying the next Ascension level.
- **5 subzones per fracture zone**: More than standard zones (3-4 subzones), providing more content density per chapter.
- **Cap zone cycling**: Only the highest unlocked zone cycles; all previous fracture zones advance forward. This allows players to choose their farming spot (highest zone they can sustain vs. a lower zone for speed).

## Act 2 Ferryman Era: Retiring Hope for a Three-Yard Reckoning

**Logged retroactively** — this decision shipped in PR #654 (d39ad67), before the design-iteration skill existed to log it at the time. Recorded now because the 2026-07-04 dossier refresh (`docs/dossiers/act2-pilgrimage.md`) flagged it as the resolution to that dossier's open question #1.

**Problem**: the Ferryman era (spec 9) originally shipped with two Reckoning purchases (Drive, Shipwright) plus a carried-over Hope gauge as the era's "second gauge" / attrition pressure. Balance sim evidence showed Hope pinned at its maximum (10/10, floor of 7) under every attentive play strategy across 24 of 25 runs — it never engaged, contributing nothing to the decision or the felt stakes.

**Options considered**:
| Option | Effect |
|---|---|
| Tune Hope's thresholds tighter | Might make it engage, but patches a gauge nobody was reading rather than fixing why it never mattered |
| Redesign Hope's mechanic | More engineering for a system with no proven identity |
| **Retire Hope, replace with a Ward yard** (chosen) | Removes the dead gauge; makes the dark's toll a per-day rate all three yards (Drive, Shipwright, Ward) now visibly bear on |

**Decision**: Retire Hope entirely (`HOPE_MAX`, `LAUNCH_HOPE`, `HOPE_FLOOR_STEADY`, Press-the-helm, Hard Rations — all removed). Add a third yard, **Ward** (`ward_level`, cost `5×1.45^L` Salvage), that multiplies the dark's toll rate down (`WARD_DECAY` 0.72 per level, floored at `WARD_TOLL_FLOOR` 0.12 — never zero). Change the dark's toll from a flat per-crossing tax to a **per-day** rate (`DARK_TAKES_PER_DAY` 0.0006, compounding over the crossing's length), so Drive (fewer days) and Shipwright (fewer crossings) also now reduce the total toll, not just Ward.

**Expected outcome** (from the commit message and `CLAUDE.md`): balanced play ~88% of the world saved across ~19–24 crossings/~3 months; reckless Drive-only or Shipwright-only play traps at ~70–74%; leaning on Ward pushes higher (~94%) at the cost of a longer era. Verified in this refresh via `ferryman_tests::strategy_sweep`: 88.1% (balanced, 24 crossings), 70.5%/74.2% (the two naive traps), and 94.3% at 32 crossings for a Ward-leaning line — matching the expectation, though the Ward-lean figure runs longer (~5 months) than the era's stated "~3 months," which the current refresh flags as a new open question rather than a miss (see dossier Open Questions #5).

## Act 2 Ward Pacing: Keep the Long Line as an Intended Branch

**Why now**: the 2026-07-04 dossier refresh measured a Ward-leaning spend policy at 94.3% souls saved but ~32 crossings / ~5 real months — beyond the era's stated "~19–24 crossings, ~3 real months" and past the `ferryman_tests` era gate's 15–30 band (not itself broken, since the committed test only exercises the balanced policy).

**Options considered**: keep as an intended "go slower, save more" branch and restate the language; tighten `WARD_COST_GROWTH` so no viable line can exceed ~24-25 crossings; or just widen the test's band without touching intent.

**Decision**: Keep it as an intended branch. `src/vessel/CLAUDE.md` now states the balanced line ("~88% saved, ~3 real months") alongside the Ward-heavy line ("~94% saved, ~5 real months") as two valid skilled outcomes, not one target with a miss. No constants changed.

**Expected outcome**: players who read "leaning on Ward saves more souls" and commit to it should feel that tradeoff as a deliberate, wide, legible choice (a slower, more careful era) rather than the era silently overrunning its own promise. Next refresh's retrospective: check whether real playtesting (once Act 2 is previewed via `QUEST_ACT2=1` in an actual session, not just the sim) treats the ~5-month line as a satisfying alternate route or as the era overstaying its welcome.

## Act 2 Ward Pacing, Re-confirmed After the 3-Month Retune (2026-07-13)

**Why now**: the 2026-07-12 era retune (`act2-era-pacing-3mo`: `CAP_GROWTH` 1.36→1.46, `DARK_TAKES_PER_DAY` 0.0006→0.0007) moved every policy line; the ward-lean branch now measures **~44 crossings / ~7.2 real months / ~93.5% saved** (committed, CI-asserted policy in `ferryman_tests::strategy_sweep_holds_the_campaign_envelope`) against the balanced line's ~22 / ~3.1 / ~88.6%. The release-checklist question (#734, 1c-3): accept, retune the Ward price ladder toward the older ~5-month note, or defer.

**Decision** (per direction, 2026-07-13): **accept as-is** — the prior "intended branch" ruling stands at the new numbers. The branch's identity is "go slower, save more," the margin is wide and legible (+4.9 points over balanced for ~2.3× the era length), and the envelope gate pins it (≥90% saved, longer than balanced). No constants changed.

**Expected outcome**: same retrospective hook as the original ruling — once real players run ward-heavy eras post-flip, check whether ~7 months reads as a deliberate pilgrimage or as overstaying; the lever, if ever needed, is `WARD_COST_GROWTH`.

## Act 2 Discovery Drought: Accepted as Intentional

**Why now**: the dossier has flagged since its first refresh that the ferry era (crossing 2+) reveals only 6 district population thresholds as new nouns across ~19-32 crossings, versus the maiden voyage's constant stream of new weather/nights/souls/rumors/refits/letters.

**Options considered**: accept the front-load-then-flatten curve as intentional; add a new mid-era noun (a soul, a threat, a rare event at specific crossings); or add lightweight per-crossing flavor-text variation without new mechanics.

**Decision**: Accept it as intentional. The maiden voyage is deliberately the decision-rich, discovery-dense half of the act; the ferry era is a hands-off victory-lap glide with a rising number, matching the "earned ramp, then fast-fun stretch" framing already present in spec 9. No new mid-era content planned.

**Expected outcome**: players should read the ferry era's flatness as a deliberate tonal shift (from active pilgrimage to a passive victory lap) rather than as the game running out of ideas. Next refresh's retrospective: if playtesting surfaces the ferry era as boring rather than restful, revisit — this decision assumes the rising numbers (souls delivered, districts, hold size) carry enough of their own momentum without new nouns.

**Superseded the same day**: see "Act 2: World Milestones" below. The designer asked for actual construction work rather than a second round of decisions, so this was revisited and built after all.

## Act 2 Launch Transition: Keep the Single Screen

**Why now**: spec 4 (`2026-03-27-vessel-mode-transition-design.md`) designed a 5-beat cinematic transition for the launch moment (Zone 50 kill → burn → Voyage begins); only a single static confirmation screen shipped.

**Options considered**: build the full 5-beat transition, matching the ceremony of the act's biggest single moment; or keep the current single screen.

**Decision**: Keep the single screen. It's a one-time, ~30-second moment in an act that is still dark-shipped, and the existing anticipation instruments (ticker whispers, the fuel bar) already do the emotional work leading up to it. Low payoff-per-effort relative to the other open items in the act.

**Expected outcome**: no measurable balance impact — this is a pure feel/ceremony question. Next refresh's retrospective: revisit if Act 2 approaches being flipped on for real (`ACT2_ENABLED = true`), since a launch-day audience will experience this moment once, for real, in a way sim runs and dossier refreshes don't capture.

**Superseded the same day**: see "Act 2: The 5-Beat Launch Transition" below. The designer asked for actual construction work rather than a second round of decisions, so this was revisited and built after all.

## Act 2: World Milestones — a Second Discovery Axis for the Ferry Era

**Why now**: immediately after "Act 2 Discovery Drought" was resolved as "accept as intentional," the designer clarified they wanted actual implementation work, not another round of settled decisions. Revisited the same question with a concrete build instead.

**What shipped**: `WorldMilestone` (`src/vessel/colony.rs`) — five milestones (10%/25%/50%/75%/90% of `INITIAL_SOULS` gone from the dying world, delivered or lost to the dark) that fire an authored log moment each, exactly once, in order. Deliberately the *other side* of the race from districts: districts mark the colony's growth (population, a rising number), milestones mark the old world's decline (a falling one). Both are pure functions of state — nothing is stored as "already fired"; `deliver_crossing()` diffs before/after and now returns a `CrossingDelivery { new_districts, new_world_milestones }` struct instead of a bare `Vec<District>`.

**Why this counts as a genuinely new noun, not districts twice**: because a milestone is keyed to `souls_remaining` rather than population, it lands on a different crossing depending on spend policy — a Ward-heavy line and a Shipwright-heavy line pass "half gone" at different points in the era, whereas district thresholds are pinned to the same population regardless of strategy. Verified with a dedicated integration test asserting all five milestones fire exactly once, in order, spread across a full era run.

**Expected outcome**: the ferry era's discovery-cadence rubric score should move up from its "front-load then flatline" 3/5 — re-scored to 4/5 this session, held below 5 because both new-noun axes are still text-only log moments rather than new mechanical levers. Next refresh's retrospective: check whether real (non-sim) play experiences these as a meaningful second thread of "what's new" or as indistinguishable from district log spam.

## Act 2: The 5-Beat Launch Transition

**Why now**: same designer clarification as World Milestones above — revisited "keep the single screen" with an actual build.

**What shipped**: `src/vessel/transition.rs` (new) — spec 4's five beats (Farewell/Unweaving/Construction/Launch/Void) as a full-screen, Enter-advanced sequence, static text per beat (per the original spec's own allowance — no character-scatter animation was added), rendered by `ui::vessel_scene::render_launch_transition()`. Gated by a new persistent `GameState::vessel_transition_played` flag; `main.rs`'s `'game_loop` shows the transition instead of the Voyage until it completes, then falls through to normal Voyage init. An interrupted transition (game closed mid-sequence) simply restarts at beat 1 next launch — the beat counter itself is transient, only "has this ever completed" is durable. One small addition beyond the original spec: a `"N / 5 — <heading>"` marker in the corner so a fixed-length sequence reads as a sequence, not an indefinite loading screen.

**Expected outcome**: no balance impact (pure ceremony), but should raise the act's biggest single moment (Zone 50 kill → burn → Voyage begins) from a bare confirmation screen to a fitting sendoff for everything the burn represents. Next refresh's retrospective: revisit once Act 2 is previewed in a real session (`QUEST_ACT2=1`) rather than only via snapshot tests — a one-time cutscene lands differently played than read as a frame dump.

## Backported Decisions (pre-OpenSpec design docs)

The following decisions were recovered from pre-OpenSpec design docs (combat-balance, balancing guide, The Deep, the Loom, the Vessel launch gate and souls, and several feature specs) that have since been folded into OpenSpec and archived under `openspec/changes/archive/`. They are grouped by system. Where a source doc's numbers later shipped differently, the note says so.

### Zone-Based Static Enemy Scaling (the combat overhaul foundation)

The single largest combat call: enemy HP/damage/defense come entirely from a fixed per-zone table (`ZONE_ENEMY_STATS`) keyed on `zone_id`, subzone depth, and boss tier — player stats are no longer an input to enemy generation. Bosses multiply the static base; an enemy `defense` field was added (variance-free), and damage changed from `saturating_sub` (floor 0) to `max(1, attacker - defender)` so high defense can never fully negate a hit. Dungeon enemies scale to the zone where the dungeon was discovered (`zone_id` stored on the `Dungeon` struct, `serde(default)` = Zone 1 for old saves).

**Design targets** (at-level player, no equipment):

| Encounter | Fight duration | Win rate |
|---|---|---|
| Normal mob | 5-8 s (3-5 exchanges) | ~95% |
| Subzone boss | 10-15 s | ~60-70% |
| Zone boss | 15-25 s | ~30-40% |
| Dungeon elite | 8-12 s | ~80% |
| Dungeon boss | 12-20 s | ~50-60% |

**Boss multipliers** (applied to the static base):

| Boss | HP | Dmg | Def | Atk interval |
|---|---|---|---|---|
| Subzone boss | 2.5x | 1.3x | 1.5x | 1.8 s |
| Zone boss | 4.0x | 1.6x | 2.0x | 1.5 s |
| Dungeon elite | 1.5x | 1.2x | 1.3x | 1.6 s |
| Dungeon boss | 2.5x | 1.4x | 1.5x | 1.4 s |

**Why**: the old relative model (`Enemy HP = player_max_hp * variance * zone_mult`) meant enemies rose in lockstep with the player, so leveling, gear, and prestige produced no real combat power, and boss multipliers compounding on an already player-matched base made zone bosses mathematically unwinnable (~0.1% success). Static stats create a genuine curve — under-leveled content is hard, over-leveled content is trivial — which is the foundation the prestige loop needs to feel rewarding, and directly fixes the "65% of P0 players stuck in Zone 1" problem.

**Alternatives considered**: keep the `player_max_hp`-relative formula (rejected — the root cause of unwinnable bosses and uniform fight duration); merely lower the player-relative multipliers (an earlier architecture-doc cut, superseded by multipliers-on-static-base); a percentage-mitigation defense model instead of flat subtraction (flat subtraction with a min-1 floor fit the existing design better).

**System**: combat (`ZONE_ENEMY_STATS`, boss multipliers, `combat/`).

### Prestige Combat Bonuses Bypass the HP-Scaling Treadmill

Prestige grants four **flat, attribute-independent** combat bonuses computed from rank alone (a `PrestigeCombatBonuses` struct), not percentages folded into `DerivedStats`:

- `flat_damage = floor(2.0 * rank^0.6)`
- `flat_defense = floor(1.0 * rank^0.55)`
- `flat_hp = floor(5.0 * rank^0.5)` — applied **combat-only**, never added to `DerivedStats.max_hp`
- `crit = min(rank * 0.5, 10.0)%`

Prestige flat bonuses and Haven percentage bonuses stack **additively** on different pipeline stages; flat damage is added after the Haven % multiplier but **before** the crit multiplier.

**Why**: anything reflected in `DerivedStats` (including `max_hp`) is exactly what enemy generation reads, so attribute- or percentage-based prestige bonuses get scaled against and neutralized. Flat, rank-derived bonuses sit outside enemy generation and therefore break the treadmill, giving even P1 a tangible edge and fixing the "prestige 1-9 feels unrewarding" dead zone before Haven unlocks at P10. Crit is capped at 10% specifically so prestige does not overshadow Haven Watchtower's +20% at T3. Sub-1 exponents mirror the XP-multiplier's diminishing-returns shape so late game does not run away. Adding flat damage before the crit roll lets crits amplify it, deliberately making crits feel more impactful for prestiged players.

**Alternatives considered**: percentage bonuses folded into `calculate_derived_stats()` (rejected — a high-risk leak that re-inflates enemy stats); multiplicative stacking with Haven (rejected — exponential runaway); adding flat damage after the crit multiplier (rejected — crits would not reward prestige investment).

**System**: combat / character (`character/prestige.rs`, `PrestigeCombatBonuses`).

### P0 Kept Intentionally Hard as a Difficulty-Taught Tutorial

P0 (no prestige combat bonuses) is left as the hardest baseline, with the first prestige providing a deliberate, noticeable combat jump (+2 dmg / +1 def / +5 HP plus the XP multiplier).

**Why**: making P0 hard and P1 clearly easier teaches, through difficulty, that prestige is the intended progression mechanic, and creates an "aha moment" after the first prestige — estimated Zone-1 clear rate lifts from ~35% (P0) toward 80%+ (P1). Players who feel stuck are being pointed at the prestige button, which is reachable (level 10) even with frequent losses.

**Alternatives considered**: flatten P0 so players can clear Zone 1 without prestiging (rejected — would obscure prestige as the core loop and remove the reason to prestige).

**System**: combat / character.

### Soften the Boss-Death Penalty Instead of a Full Kill-Counter Reset

Reduce the friction of dying to a boss rather than resetting `kills_in_subzone` to 0 and forcing 10 fresh kills. The shipped form (`boss_retry_kills`) drops the requirement from 10 to 5 for the retry, then resets to 10 on kill or zone change.

**Why**: with static scaling, zone bosses are legitimately beatable but at only a 30-40% win rate; a full 10-kill grind-back (75-105 s) punishes players for attempting challenging content after a close loss. A softer penalty keeps players engaged rather than frustrated while retaining some cost via the regen timer.

**Alternatives considered**: full preservation so the boss respawns immediately (idle-RPG friction norm, rejected as too soft); half-preservation (kills/2); the full 10-kill reset (rejected as too punishing at a 30-40% win rate).

**System**: combat / zones.

### Beta Rebalance: Tune Constants, Not the Enemy Architecture

The five beta-test balance issues were fixed purely by changing values in `src/core/constants.rs` (`ZONE_ENEMY_STATS`, boss/dungeon multipliers, prestige-bonus formulas) — no changes to enemy generation or combat logic. Concretely: roughly double early-zone HP (Zone 1 ~30→~55-60) and raise enemy damage ~1.4-2x; compress the Zone 7-10 curve so the Zone 8→10 wall shrinks from ~2x to ~1.4x; scale prestige combat bonuses ~3-5x while keeping the sub-1 diminishing-returns exponents; raise dungeon multipliers to hit a 10-20% failure rate; and raise the prestige crit cap from 10% to 15%. Zone/dungeon tuning was landed **before** prestige-bonus tuning, not simultaneously.

**Why**: all five findings traced to one root cause — the zone stat tables and prestige formulas modeled theoretical "at-level" players and overestimated difficulty. The static-stats architecture was sound; only the tuning parameters were wrong. Constant-only changes also avoid save migration (enemy structs regenerate on spawn, prestige bonuses recompute from rank). Landing zone balance first let beta testers validate the difficulty floor before prestige bonuses layered on top, isolating each change's effect even though both edit the same file. The 10% crit cap was reached around P20, giving P30+ players no further crit reward — 15% keeps crit meaningful deeper.

**Alternatives considered**: re-architect enemy scaling or reintroduce player-HP-scaled enemies (rejected — the architecture was correct); raise prestige bonuses without touching zone stats (rejected — the zone jump dwarfed any prestige gain); ship all changes in one merge (rejected — would confound beta feedback). The two source docs disagreed on exact final tables/exponents; the reconciled values are in "Key Constants" in CLAUDE.md.

**System**: combat (`core/constants.rs`).

### 3D ASCII Combat: Enemy Scale Driven by the HP-Ratio Differential

The optional 3D ASCII combat renderer sizes the enemy sprite from a `combat_depth = 0.5 + (player_hp_ratio - enemy_hp_ratio) * 0.3`, clamped to `[0.2, 0.9]`, mapped to sprite height `min_height(3) + 17 * depth` (3-20 lines) with three-band depth shading (`depth < 0.3` far/lighter, `< 0.7` medium, else close/denser) over an ASCII density gradient. Compositing is a fixed seven-layer back-to-front pipeline (ceiling → walls → floor → atmosphere → enemy sprite → effects → UI overlay). Effect timings are aligned to combat cadence (player/enemy attack sequences 1.5 s, death + HP regen 2.5 s).

**Why**: tying visual distance to the HP differential makes the enemy loom larger when the player is losing and retreat when winning, so combat state is readable through scale alone; the clamp keeps the enemy visible at both extremes. Discrete shading bands give cheap, deterministic depth perception without geometry math. A defined back-to-front order gives correct painter's-algorithm compositing and enables per-layer caching. Aligning effect lengths to the real attack/regen intervals keeps animations in step with the combat state machine instead of desyncing.

**Alternatives considered**: a fixed enemy size or a non-HP signal such as elapsed time (drops the built-in who's-winning readback); continuous per-pixel shading (more expensive, harder to keep deterministic); ad-hoc single-pass drawing (breaks occlusion and per-layer caching).

**System**: combat (3D ASCII renderer, UI).

### 3D ASCII Combat: 10 FPS Budget and an Accessibility Fallback

Render to a 10 FPS / 100 ms-per-frame budget (matching the game's existing tick), achieved by pre-rendering/caching static geometry, caching scaled sprites at common sizes, pooling effect vectors, redrawing only changed layers, and building frames with `Vec<String>` rather than string concatenation. The whole 3D view is optional (`Combat3DConfig`): when disabled it falls back to the simple emoji view, and screen shake, red/white flashes, effect density, and sprite detail are each individually disableable.

**Why**: the renderer must fit inside one 100 ms tick to avoid a gameplay performance regression; caching static geometry and reusing allocations keeps per-frame cost inside the budget. Screen shake and flashes are accessibility/epilepsy hazards and the full effect stack has a cost, so each must be individually disableable and the whole view must have a lightweight escape hatch.

**Alternatives considered**: re-render all layers every frame and concatenate strings (blows the budget, allocates on the hot path); ship the 3D view unconditionally with all effects always on (fails epilepsy safety, no escape hatch on slow terminals).

**System**: combat (3D ASCII renderer, UI, accessibility).

### Auto-Equip Uses Intrinsic `power()`, Not Character-Weighted `score_item()`

`auto_equip_if_better()` compares items by `item.power()` (character-independent, equal attribute weights plus weighted affix values — the same number shown by the UI's lightning-bolt) instead of the old `score_item()`, which weighted attributes toward the character's current distribution. The now-dead `score_item()` and `calculate_attribute_weights()` were deleted; `affix_power_weight()` was kept because `power()` still uses it.

**Why**: auto-equip now matches player expectations — it equips the item whose displayed power is higher. `score_item()` could equip a weaker item over a stronger one (reading as a bug) because attributes are assigned randomly on level-up, so specialization weighting was solving a problem that doesn't meaningfully exist.

**Alternatives considered**: keep `score_item()`'s attribute-specialization weighting (rejected — hidden weighting produced surprising, non-predictable equips for little real value given random attribute distribution). The trade accepted is that a DEX-heavy character no longer prefers DEX items — specialization traded for predictability and simpler code.

**System**: items.

### Character Persistence: JSON, One Self-Contained File Per Character, Atomic Writes

Each character is stored as its own `~/.quest/{sanitized_name}.json` file — no index/metadata file — with a generated UUID `character_id` separate from the mutable `character_name`. Saves are atomic (write `.{name}.tmp`, then rename; keep the old file on any failure). Names are sanitized to filename-safe tokens with numeric-suffix collision resolution (`test`→`test_2`) and a UUID fallback for empty results. The legacy binary save is migrated non-destructively (imported as a new character, original left in place). Concurrent instances are allowed with last-write-wins (no lock files). The roster is capped at 3 characters; deletion requires typing the exact character name (case-sensitive).

**Why**: JSON is human-readable, debuggable, and portable — copying a `.json` equals copying a character. A UUID gives a stable identity that survives renames even though filenames derive from the name. Atomic temp-then-rename prevents a crashed or full-disk write from destroying the most recent good save. Non-destructive migration preserves existing progress with zero risk. Multi-instance is a rare edge case for a single-player game, so lock files weren't justified. Exact-name deletion prevents accidental loss of a high-investment character.

**Alternatives considered**: a single file containing all characters, or a separate index file (rejected — metadata-sync issues); using the name as sole identity (breaks continuity on rename); write-in-place (risks corruption); lock files for concurrency (complexity not justified); auto-delete the old save after migration (rejected — let the user decide). Note: the design doc also called for a SHA256 checksum over all fields; per "Save Format: Binary vs JSON" above, that was ultimately dropped as unimplemented.

**System**: persistence (`character/`, `~/.quest/*.json`).

### Character Titles: Account-Wide, Curated to 64 Achievements, Player-Chosen

The selected title is `selected_title: Option<AchievementId>` on the account-wide `Achievements` struct (in `achievements.json`), not per-character. Only a hand-picked set of 64 achievements grant titles (a static `ALL_TITLES` map), the player picks/previews/clears it in a dedicated `[T]` overlay, and it renders as a comma-separated text suffix on the name (`Evaa, Eternal`) independent of the kill-count badge icon. An invalid selection (achievement not unlocked, or not a title) is silently cleared on load; the field is `serde(default)` for pre-feature saves. Titles are shown in the stats panel/compact bar/character-select only, never in the combat scene (which uses badges) or in save filenames.

**Why**: titles are earned from account-level achievements, so ownership belongs at the account level and carries across characters. Curation keeps titles meaningful and prestige-signalling — only milestones like P100 Eternal, Storm Leviathan, and Master-tier minigames are worth wearing. Players should control which accomplishment they broadcast. Keeping the title decoupled from the badge lets a header show both; keeping filenames keyed on the raw name keeps persistence stable.

**Alternatives considered**: per-character storage; a title for every achievement or auto-generated text; auto-displaying the newest/highest title with no player choice; folding titles into the badge system; showing the title everywhere including combat and filenames. (Implementation note: the "Soulforged" +10 title maps to `AchievementId::MasterSmith`, correcting the design doc's non-existent `SoulforgeX`.)

**System**: achievements.

### Achievement NEW Badges Are Transient, Reusing the Pending-Notification Queue

The "NEW" markers use a `recently_unlocked: Vec<AchievementId>` field marked `#[serde(skip)]` (never serialized), populated by draining `pending_notifications` when the browser opens (`clear_pending_notifications()`), and cleared when the browser closes.

**Why**: NEW badges are a session-scoped UI marker; persisting them would carry stale "new" state across reloads and require migration. The pending-notification queue already captures exactly what was unlocked since the player last acknowledged, so reusing it avoids a parallel tracker. Clearing on close gives predictable "appear on open, gone on next open" semantics.

**Alternatives considered**: persist unlock timestamps and compute recency at render time (adds save-schema surface and ambiguity); a separate tracker updated in `unlock()` (duplicative); time-based expiry (fuzzier semantics).

**System**: achievements.

### Auto-Update: Opt-In CLI Command with a Fail-Closed Save Backup

Updates install only when the user runs `quest update`; startup does a lightweight background check-and-notify (on its own thread, joined after terminal setup, auto-dismissing after ~5 s) and never mutates the binary or saves. Before any download, every `*.json` in `~/.quest/` is copied into a timestamped `backups/` folder and the update **aborts** if the backup fails. Version identity is the 7-char commit hash embedded at compile time (`build.rs`) plus an ISO build date, compared against the parsed `build-<sha>` GitHub release tag. Binary replacement is OS-specific: Unix overwrites the running executable in place; Windows renames the running exe to `.old`, moves the new one in, then deletes `.old` — no self-re-exec. HTTP uses the blocking `ureq` client against unauthenticated GitHub Releases + Compare APIs. Network failure is handled asymmetrically: a failed startup check is silently swallowed, but `quest update` surfaces the error and exits non-zero.

**Why**: keeps launch fast and never changes the binary or saves without explicit intent. A fail-closed backup guarantees an update can never destroy progress. A short commit hash gives an unambiguous identity for a project with no semver scheme. In-place overwrite is safe on Unix (kernel keeps the in-use inode mapped) but Windows locks the running exe against overwrite while permitting rename. Unauthenticated public endpoints avoid shipping a token; the Compare API yields commit messages directly for a changelog. Startup must never be blocked or noisy when offline, but a user who explicitly asked to update deserves to know why it failed.

**Alternatives considered**: auto-download/install on startup; a persistent in-game updater UI; no backup or a single rolling backup; semver or full-SHA identity; an async HTTP client; self-re-exec after update; always-show or always-silent error handling.

**System**: tooling (`utils/updater`, `build.rs`).

### Balance Simulator: Extend the Existing Simulator, Inject Outcomes at the State Level

Rather than a separate balance-validation binary, the existing simulator was generalized: it drives `game_tick_with_context()` with a persistent `TickContext` (real `LoomState` alongside Haven/Enhancement/Deep/Achievements), and `inject_outcomes()` calls the game's own public functions (`ascend()`, `perform_prestige()`, `on_minigame_won()`) wherever they exist, falling back to direct mutation only for enhancement levels and sigils. PR is tracked by snapshotting `prestige_rank` before/after each tick (positive deltas → earned, negative → spent). The four Haven strategies were replaced by three player-archetype profiles (casual/optimal/speedrun) behind `--strategy`.

**Why**: the simulator already runs the game's tick loop with seeded RNG, CSV export, and multi-run support — reusing it avoids duplicating that infrastructure and keeps simulated progression faithful to real mechanics. Migrating to `game_tick_with_context()` was a prerequisite because the deprecated `game_tick()` creates a throwaway `LoomState` each tick, so Loom patterns/shuttles/WR→PR/zone-unlocks never accumulate. A single before/after `prestige_rank` delta uniformly captures every PR source and sink without enumerating `TickEvent` variants or double-counting injection costs. Three archetypes answer the core balance questions (relaxed feel, ideal-but-human, fastest).

**Alternatives considered**: a dedicated new binary (redundant); keep calling deprecated `game_tick()` (Loom state lost every tick); drive real inputs/AI players through the minigames (out of scope); run the enhancement success/failure RNG per attempt (tests the wrong thing for balance curves); an external TOML/JSON assertion config (deferred). Constant-drift detection was explicitly left out of scope — it's owned by the doc-audit and wiki-audit skills.

**System**: tooling (`bin/simulator/`).

### Chess Minigame: Player-Reviewed Challenge Menu and the `chess-engine` Crate

Chess does not auto-enter on discovery (the way fishing and dungeons do); it surfaces a `PendingChallenge` the player opens with Tab and reviews in a navigable list/detail menu. That menu is generic — it holds items keyed by an extensible `ChallengeType` enum, with chess merely the first producer. Move generation and the minimax + alpha-beta AI come from the dependency-free `chess-engine` crate. Winning grants +1 to +5 prestige ranks immediately with **no prestige reset** (level/XP/attributes/equipment/zones all kept). Difficulty is player-chosen:

| Tier | AI (random-mix + ply) | Reward | Approx ELO |
|---|---|---|---|
| Novice | 50% random, 1 ply | +1 PR | ~500 |
| Apprentice | 1 ply | +2 PR | ~800 |
| Journeyman | 2 ply | +3 PR | ~1100 |
| Master | 3 ply | +5 PR | ~1350 |

In-progress games and pending challenges are `#[serde(skip)]` (lost on quit/prestige); only aggregate `ChessStats` persists.

**Why**: chess requires active attention, so auto-entering would be hostile and the large reward means accidental entry/exit must be prevented — a reviewed menu also establishes a reusable pattern for future player-controlled minigames. The crate eliminates ~400-600 lines of hand-rolled move generation/search and gives correct castling/en-passant/promotion/draw handling with zero transitive dependencies. Reset-free prestige makes discovery exciting and rewards skill, with the +3→+5 jump at Master making the hardest tier disproportionately worth attempting; the win requirement balances the power.

**Alternatives considered**: auto-enter like fishing/dungeons; a chess-specific menu; a hand-rolled engine; normal prestige with its reset, or lower-value XP/item rewards; a single fixed difficulty or a linear reward curve; persisting mid-game state across sessions. (v1 auto-selects queen promotion and relies on the crate's stalemate detection without threefold-repetition or 50-move tracking.)

**System**: challenges.

### Chrono Surge Accelerates Deep Missions at 1:1, Reusing Existing Resolution

During a Chrono Surge, `accelerate_missions()` shifts each active Deep mission's `ends_at` (and unresolved event timestamps) by exactly the surge tick — 100 ms of surge subtracts 100 ms of mission timer — but does **not** resolve missions itself. The normal tick resolution path detects the elapsed timer and moves completed missions into `pending_results` for player review; check-in events whose timestamps fall into the past are resolved with their `auto_resolve_choice`. A `missions_completed` counter surfaces on the surge summary.

**Why**: a 1:1 ratio matches Chrono Surge's existing "fast-forward game time" semantics, keeping mission progress consistent with combat/XP/loot during a surge. Shifting `ends_at` increases `progress()` for the same wall-clock `now`, so the established resolution path fires naturally — avoiding a duplicate completion codepath. Routing completions through `pending_results` preserves the Deep overlay's narrative/loot reveal; auto-resolving passed check-ins matches offline behavior so surging and being offline produce consistent outcomes.

**Alternatives considered**: a tunable acceleration multiplier; have `accelerate_missions()` resolve/finalize missions directly (duplicates resolution logic); silently auto-resolve completions (loses the reveal); block acceleration on pending events (diverges from offline resolution).

**System**: stormglass / deep.

### Cloud Sync: Reuse git2 Time Vault, Never Clone, "Keep Both" Guarantees No Data Loss

Cloud sync is built on the existing git2 Time Vault history, using a private auto-created `quest-saves` GitHub repo as a remote (branch = character/fork mapping, so no new data model). Every machine always `git init`s the local `~/.quest` repo then adds the remote — there is **no `git clone` path**. A background push of all branches fires directly off each save commit (no debounce/timer), with only one cloud op in flight at a time (a second push is skipped, since the next commit pushes everything). Before applying a pull, each branch's `(ahead, behind)` is computed via `git2::graph_ahead_behind`: in-sync skips, behind-only fast-forwards, ahead-only skips, ahead-and-behind blocks as diverged. Genuine divergence offers Keep local / Use cloud / **Keep both** (rename local to `main-backup-<date>`, reset to cloud) / Decide later. The PAT lives in a git-ignored `.cloud.json`, injected as `https://x-access-token:TOKEN@github.com/...`. Auto-fetch on launch is non-fatal.

**Why**: reusing the proven save-history machinery means git provides branching, history, and diffing for free. A single init+remote codepath is simpler than special-casing a clone bootstrap. Save commits already fire only on significant events, so a debounce/timer would add complexity for no benefit. Fast-forwarding only where provably safe, never auto-resolving real divergence, means an automatic sync can never silently overwrite progress — and the "Keep both" option resolves even the simultaneous-play edge case with zero data loss. Keeping the PAT out of the tracked repo prevents leaking it to the remote. Network problems must never block play.

**Alternatives considered**: a bespoke cloud service or custom protocol; a public repo (save data must be private); a clone-based bootstrap (two divergent codepaths); a timer/debounce or a push queue; blanket fast-forward/reset-to-remote or always-manual resolution; force-push-vs-reset only (always discards one side); storing the token in the tracked repo; blocking startup on sync.

**System**: time-vault (`history/`, `cloud.rs`).

### The Deep: Wall-Clock Missions, Optional Engagement, Generational Persistence

The Deep runs mercenary-squad missions on **wall-clock time** (2-24 h, progressing while the game is closed), not game ticks. Engagement is optional: check-in events reward attention but never punish absence — events that fire while offline take the safe `auto_resolve_choice`, which never risks a merc. Persistence is deliberately asymmetric across prestige:

| Persists across prestige | Resets on prestige |
|---|---|
| Guild rank, cleared layers, infrastructure | Individual mercenaries |
| Intel per layer, campaign progress, trophies | Active missions, Warband Marks, merc levels/gear |

Warband Marks are the resetting currency; guild rank is persistent progression. Breakthrough missions on deep layers award fractional prestige-rank fragments as an alternate PR path.

**Why**: the hours/days timescale is a fundamentally different engagement rhythm from the seconds/minutes of combat, and the generational theme (each prestige sends a new generation deeper on infrastructure the last left behind) makes the reset feel like continuity rather than loss. Never punishing absence fits an idle game. Persisting infrastructure/rank while resetting mercs gives every prestige a fresh roster but a durable base.

**Alternatives considered**: tick-based missions (would collapse the distinct rhythm); punishing missed events; resetting everything on prestige (loses the generational payoff). Note: the shipped discovery trigger changed from the design doc's per-tick RNG roll to firing on the first Expanse cycle boss kill (The Endless) at P15+ — see "Key Constants" in CLAUDE.md.

**System**: deep (`src/deep/`).

### The Loom: Combat→Factory Transition Gated by a 168-Hour Gateway Mission

Entering the Loom requires completing the **Gateway Expedition** at Deep Layer 30 — a 168-hour (7-day) mission with a fixed duration that **bypasses all duration modifiers** (infrastructure, familiarity, saboteur, overpower); only wall-clock time and Chrono Surge apply. The game then transitions from combat RPG to incremental factory/engine-builder, with the two loops **coexisting** (no hard cutoff): early Loom stages need prestige fuel from combat, and the crossover — when the Loom out-produces combat — is a key milestone. Existing systems (Deep/Haven/Stormglass/Ascension) feed the Loom with diminishing ongoing bonuses (meaningful when it's new, negligible once it scales), and integration is **one-directional** (existing systems feed the Loom, not the reverse) with the door left open for a future bidirectional beat.

**Why**: a week-long fixed-duration gate makes entering the endgame a deliberate, unhurried commitment that can't be shortcut by the Deep's own optimization levers. A hybrid coexistence lets players migrate naturally from combat to factory without a jarring mode switch. Diminishing existing-system bonuses acknowledge the player's journey without distorting the Loom's own curve. One-directional integration keeps the Loom's balance self-contained.

**Alternatives considered**: a modifier-affected Gateway (would let players trivialize the gate); a hard cutover from combat to factory; existing systems providing flat (non-diminishing) bonuses (would distort the Loom curve); bidirectional integration at launch (deferred). Note: the shipped Loom economy diverged from this doc's six-resource cyclical engine toward direct-pull refineries and the WR→PR conversion — see the Loom entries in "Key Constants" and `src/loom/CLAUDE.md`.

**System**: loom (`src/loom/`).

### Vessel Launch: Hold 250,000 PR for a Single All-or-Nothing Burn

The Act 2 launch gate is simply **holding 250,000 prestige rank at the moment of launch** — there is no fuel accumulator, no partial banking, and no diversion of PR grants (WR→PR, Power Cores, challenges all keep ticking untouched). Launch is one confirmed action that deducts the full 250,000 at once, leaving `rank - 250,000` behind. The gate `can_launch()` also requires the Z50 signal discovered, Ascension X, and all 28 Woven Patterns. The whole feature ships **dark** behind a compile-time kill-switch (`ACT2_ENABLED = false`, `QUEST_ACT2=1` to preview), but Z50 detection still silently records `vessel_signal_discovered` in saves so qualified players light up the instant Act 2 is enabled.

**Why**: the hero fights at full prestige strength for the entire wait (rank and its bonuses stay intact until the burn), a veteran already holding 250k+ can launch the moment the signal appears, and the model is the simplest possible one — no PR grant site is touched, there is no partial state to persist. Above all it produces **one dramatic moment** instead of many small ones: the player watches everything they accumulated vanish in a single confirmed action. That is the launch.

**Alternatives considered**: a `vessel_fuel` accumulator with partial banking and transfer controls (rejected — more state, more code, and it dilutes the single-burn drama); freezing rank or diverting PR grants during the wait (rejected — would touch five grant sites and weaken the hero during the climb).

**System**: vessel (`src/vessel/`, launch gate).

### Act 2 Souls: The Covenant — Nothing Harms a Soul While You Are Away

A hard, CI-enforced invariant: **no tick-driven code path may reduce the roster.** Nights, weather, drift, hold-station, and offline resolution never touch souls; a property test simulates arbitrary offline windows and asserts roster count is invariant. Souls are lost **only in authored scenes** attached to named threats — the threat was on the junction card, the road was chosen, and the scene offered a priced alternative. There is no dice-based loss. A loss is memorialized: the soul's name is carved into the hull art for the rest of the game (Act 3 included), their arc becomes a memorial manifest line, and their counsel lines go silent.

**Why**: Act 2's covenant with the player is that stepping away is always safe, so the absence-safe guarantee has to be mechanical, not just intent — hence the CI property test. Catastrophes should be priced, chosen, and become story rather than arriving as bad luck, which is why loss follows only from a stated-stakes choice. Memorializing loss (rather than hiding it) makes the emptier junction the intended emotional payoff.

**Alternatives considered**: allowing tick/offline code paths to injure or lose souls (breaks the covenant); dice-based loss on failed events (rejected — loss must be a chosen, priced consequence); silently removing a lost soul without a memorial. (Related: the Hope gauge that this doc priced loss/farewell against was later retired entirely — see "Act 2 Ferryman Era: Retiring Hope" above — so `LOSS_HOPE_COST`/`FAREWELL_HOPE_COST` are gone, but authored-only, memorialized loss remains.)

**System**: vessel (Act 2 souls, `src/vessel/voyage.rs`).

### Act 2 Souls: Affinity as the Single Aptitude Axis; 8 Souls Against 7 Berths

A soul has exactly one aptitude axis — **affinity** (Helm, Tender, Watch, or none) — which does double duty: it strengthens the matching station's effect and it is what the night system reads (no separate per-soul night-suitability table). There are **8 authored souls** (3 board at launch, 5 found on the route) against **7 berths**, so a player who says yes to everyone faces exactly one farewell. Every recruitable soul has a site on each branch arm, so **every route meets every recruitable** — different scenes, same person (content-parity: different routes never means fewer souls). Two souls have no affinity at all (their value is counsel and their arcs). Arcs advance only in **rest days** — a soul at a station is not resting — so coverage and story are a standing trade-off. There are no soul stats, levels, equipment, upkeep, or procedural generation.

**Why**: collapsing aptitude to one axis keeps the soul model small and legible (the parent spec's "small numbers, no hidden arithmetic" pillar) and makes affinity meaningfully load-bearing in two systems at once. 8-against-7 forces exactly one real roster decision without inventing upkeep. Content-parity guarantees route choice never costs the player cast. Making arcs cost rest days means the coverage-vs-story tension is the reason a 7-berth roster matters. No-affinity souls prove stations aren't the only reason to want someone aboard.

**Alternatives considered**: separate stat/level/equipment systems or per-soul morale (rejected — hope is one shared gauge; souls carry only voice beyond the triangle); a separate per-soul night table (redundant with affinity); procedural souls (rejected — eight authored people); a fourth "Keel" station for threat protection (cut — threat pricing belongs to the road card and scenes, not a passive slot); upkeep costs (souls eat nothing — the berth question is always *who*).

**System**: vessel (Act 2 souls, stations, arcs).

### Six-Attribute Model: Random +3 per Level, Prestige-Scaled Cap, CHA Amplifies the Prestige Multiplier

Six attributes (STR/DEX/CON/INT/WIS/CHA) each use `modifier = (value - 10) / 2` (integer division, min 0). On level-up the character gains **+3 random points** distributed among non-capped attributes; the cap is `20 + 5 * prestige_rank`. Each attribute drives one lever, including CHA which feeds the prestige multiplier itself:

| Attribute | Effect | Per modifier point |
|---|---|---|
| STR | Physical damage | +2 damage (`5 + STR_mod * 2`) |
| DEX | Defense and crit | +1 defense, +1% crit (`5% + DEX_mod`) |
| CON | Max HP | +10 HP (`50 + CON_mod * 10`) |
| INT | Magic damage | +2 damage (`5 + INT_mod * 2`) |
| WIS | XP gain | +5% XP (`1.0 + WIS_mod * 0.05`) |
| CHA | Prestige multiplier | `effective = base + CHA_mod * 0.1` |

**Why**: random point allocation keeps builds varied without asking an idle-game player to make per-level micro-decisions, while the prestige-scaled cap creates a "soft ceiling" at each tier (at the P0 cap of 20, characters converge to the same stats past ~level 25) that specifically encourages prestiging. CHA feeding the prestige multiplier gives a sixth attribute a meaningful role in the core loop rather than being dead weight.

**Alternatives considered**: player-directed attribute allocation on level-up (rejected — too much per-level friction for an idle game); a flat (non-prestige-scaled) attribute cap (rejected — removes the soft ceiling that motivates prestige). Note that because random allocation makes attribute distribution unpredictable, downstream systems avoid character-attribute-weighted logic (see "Auto-Equip Uses Intrinsic `power()`" above).

**System**: character / combat (attributes, `character/`).

### XP Curve: 100 × level^1.5, Chosen for Idle Pacing

Levels require `xp_needed = 100 * level^1.5` (`XP_CURVE_BASE = 100.0`, `XP_CURVE_EXPONENT = 1.5`), and kill XP is the only source (`random(200..=400)` per kill). The exponent is treated as a primary tuning lever:

| Exponent | Effect |
|---|---|
| 1.3 | Faster leveling, shorter prestige cycles |
| **1.5** | **Current** — balanced idle pacing |
| 1.7 | Slower leveling, more grind per prestige |
| 2.0 | Very slow — hardcore only |

**Why**: a 1.5 power curve keeps early levels quick (hooking new players and giving fast post-prestige cycles) while making high levels a meaningful grind, matching the intended prestige pacing (P1 in 30-60 min, later cycles stretching to days). Sourcing all XP from kills (200-400 per kill) keeps combat the significant driver of progression rather than passive time, which is what lets active play stay ahead of pure idle.

**Alternatives considered**: a lower exponent like 1.3 (faster but trivializes prestige cycles) or higher like 1.7-2.0 (too grindy outside a hardcore mode); passive/tick-based XP as a co-equal source (rejected — would make combat feel unrewarding versus idling). The exponent is flagged as high-impact: changing it ripples through all level timings and prestige pacing.

**System**: core / character (XP economy, `core/constants.rs`).

### Balance Philosophy: Active Play ~2-3x Idle, Endgame in Weeks Not Hours

The guiding balance targets for the whole economy: meaningful AFK progress, but active decisions (prestige timing, minigames, Haven) should be ~2-3x more efficient than pure idle ("the golden ratio"); no hard walls (progress slows but never stops); and each prestige should feel like a real power boost. The intended pacing:

| Milestone | Target time | Feel |
|---|---|---|
| First prestige (P1) | 30-60 min | "I get it now" |
| Haven unlock (P10) | 8-12 hours | "New system!" |
| Stormbreaker | 2-4 weeks | "Finally!" |
| The Expanse cycles | Infinite | "One more run" |

The balancing guide also codifies **danger zones** — constants not to touch without simulation (`TICK_INTERVAL_MS`, `BASE_XP_PER_TICK`, zone/prestige level requirements, `MAX_FISHING_RANK`) — versus safe-to-tune levers (fish weights, enemy names, item affix ranges, dungeon room types, UI).

**Why**: writing the golden ratio and the milestone/feel targets down gives every future tuning change a yardstick — a change is "right" if it keeps active play ahead of idle without creating a hard wall and preserves the prestige power-boost feel. The danger-zone/safe-lever split protects the load-bearing constants (which ripple through the entire XP and progression economy) from casual edits while marking what can be freely adjusted for flavor.

**Alternatives considered**: leaving pacing implicit in the constants (rejected — no shared yardstick for whether a change helps or hurts); treating all constants as equally safe to edit (rejected — the danger-zone constants ripple through everything and need simulation before changing).

**System**: (cross-cutting balance philosophy).
