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

Not all challenges are equally discoverable:

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

**Decision**: Extract the per-tick orchestration function from main.rs into `src/core/tick.rs`, returning a `TickResult` struct with `Vec<TickEvent>` (now 48 variants) instead of mutating UI state directly.

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
