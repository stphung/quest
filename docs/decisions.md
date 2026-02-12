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

**Decision**: Add a `src/bin/simulator.rs` binary that runs the game tick loop headlessly, collecting metrics for game balance analysis.

**Rationale**: Balance testing previously required playing the game manually or writing one-off test harnesses. The simulator reuses the exact same `game_tick()` function, ensuring perfect fidelity with the real game. It supports:
- Configurable tick count, RNG seed, starting prestige
- Multi-run aggregation with min/avg/max statistics
- CSV time-series export for graphing progression curves
- Verbose per-tick event logging for debugging

This enables systematic balance validation: "does a P0 character reach Zone 2 in 1 hour?" or "what's the item drop distribution over 10,000 ticks across 100 seeds?"
