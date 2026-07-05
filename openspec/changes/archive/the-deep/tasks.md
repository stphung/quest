> Backported implementation plan (completed — this work shipped).

## 2026-02-24-deep-narrative-implementation-plan.md

# The Deep Narrative — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Sealed Root narrative arc for The Deep — Rift Resonance discovery chain, per-tier hub atmosphere messages, and the Gateway at Layer 30.

**Architecture:** New persistent fields in `DeepPersistent` drive a 4-stage story chain gated by Rift Resonance (incremented on qualifying prestige). Layer 30 is a special story layer with a unique `GatewayExpedition` mission type. Hub atmosphere messages are keyed to frontier layer tier. A `GatewayOpened` achievement marks completion.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (terminal UI), Chrono (wall-clock time)

**Design doc:** `docs/plans/2026-02-24-deep-narrative-design.md`

---

## Task 1: Add Rift Resonance and Story Chain Fields to DeepPersistent

**Files:**
- Modify: `src/deep/types.rs` (DeepPersistent struct, ~line 639)
- Test: `tests/deep_prestige_persistence_test.rs`

**Step 1: Add fields to DeepPersistent**

In `src/deep/types.rs`, add these fields to `DeepPersistent` after `generation_counter`:

```rust
    /// Rift Resonance — increments each prestige where player reached The Expanse.
    #[serde(default)]
    pub rift_resonance: u32,
    /// Story chain progress (0 = not started, 1-4 = stages, 5 = discovered).
    #[serde(default)]
    pub deep_story_stage: u8,
    /// Rift Fragments collected (0-4).
    #[serde(default)]
    pub rift_fragments: u8,
    /// Whether the Gateway at Layer 30 has been opened.
    #[serde(default)]
    pub gateway_opened: bool,
```

Update `DeepPersistent::new()` to initialise these fields:

```rust
    rift_resonance: 0,
    deep_story_stage: 0,
    rift_fragments: 0,
    gateway_opened: false,
```

**Step 2: Write serde backward compatibility tests**

In `tests/deep_prestige_persistence_test.rs`, add:

```rust
#[test]
fn test_deep_persistent_rift_resonance_defaults_on_missing() {
    let json = r#"{"discovered":false,"guild_rank":1,"guild_upgrade_cost":0,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0}"#;
    let persistent: DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(persistent.rift_resonance, 0);
    assert_eq!(persistent.deep_story_stage, 0);
    assert_eq!(persistent.rift_fragments, 0);
    assert!(!persistent.gateway_opened);
}

#[test]
fn test_deep_persistent_rift_resonance_roundtrip() {
    let mut persistent = DeepPersistent::new();
    persistent.rift_resonance = 7;
    persistent.deep_story_stage = 4;
    persistent.rift_fragments = 3;
    persistent.gateway_opened = true;

    let json = serde_json::to_string(&persistent).unwrap();
    let loaded: DeepPersistent = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rift_resonance, 7);
    assert_eq!(loaded.deep_story_stage, 4);
    assert_eq!(loaded.rift_fragments, 3);
    assert!(loaded.gateway_opened);
}
```

**Step 3: Run tests**

Run: `cargo test --test deep_prestige_persistence_test`
Expected: All pass including new tests.

**Step 4: Commit**

```bash
git add src/deep/types.rs tests/deep_prestige_persistence_test.rs
git commit -m "feat(deep): add rift_resonance, story chain, and gateway fields to DeepPersistent"
```

---

## Task 2: Implement Rift Resonance Increment on Prestige

**Files:**
- Modify: `src/input/prestige_input.rs` (~lines 64-84, 136-155)
- Modify: `src/deep/types.rs` (add helper method)
- Test: `tests/deep_integration_test.rs`

**Step 1: Add `increment_rift_resonance()` to DeepState**

In `src/deep/types.rs`, add a method to `impl DeepState` (before `on_prestige()`):

```rust
    /// Increment Rift Resonance if the player reached The Expanse (Zone 11+)
    /// and is at least P15. Call BEFORE `on_prestige()` since zone data is
    /// on the character state, not Deep state.
    pub fn maybe_increment_rift_resonance(&mut self, current_zone_id: u32, prestige_rank: u32) {
        if prestige_rank >= DEEP_MIN_PRESTIGE_RANK && current_zone_id >= 11 {
            self.persistent.rift_resonance += 1;
        }
    }
```

**Step 2: Wire into prestige_input.rs**

In `src/input/prestige_input.rs`, there are two prestige paths (vault and non-vault). In BOTH paths, add this line BEFORE the `deep.on_prestige()` call:

```rust
                    deep.maybe_increment_rift_resonance(
                        state.zone_progression.current_zone_id,
                        state.prestige_rank,
                    );
```

This goes right before the existing `deep.on_prestige();` line in both the vault path (~line 74) and the normal path (~line 146).

**Step 3: Write integration test**

In `tests/deep_integration_test.rs`, add:

```rust
// =========================================================================
// Rift Resonance
// =========================================================================

#[test]
fn test_rift_resonance_increments_when_in_expanse_at_p15() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 15);
    assert_eq!(deep.persistent.rift_resonance, 1);
}

#[test]
fn test_rift_resonance_no_increment_below_p15() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 14);
    assert_eq!(deep.persistent.rift_resonance, 0);
}

#[test]
fn test_rift_resonance_no_increment_below_zone_11() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(10, 15);
    assert_eq!(deep.persistent.rift_resonance, 0);
}

#[test]
fn test_rift_resonance_accumulates_across_prestiges() {
    let mut deep = DeepState::new();
    for _ in 0..7 {
        deep.maybe_increment_rift_resonance(11, 20);
        deep.on_prestige();
    }
    assert_eq!(deep.persistent.rift_resonance, 7);
}

#[test]
fn test_rift_resonance_survives_prestige_reset() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 15);
    deep.on_prestige();
    assert_eq!(deep.persistent.rift_resonance, 1, "Rift resonance should persist across prestige");
}
```

**Step 4: Run tests**

Run: `cargo test --test deep_integration_test`
Expected: All pass.

**Step 5: Commit**

```bash
git add src/deep/types.rs src/input/prestige_input.rs tests/deep_integration_test.rs
git commit -m "feat(deep): increment rift resonance on qualifying prestige"
```

---

## Task 3: Implement Story Stage Progression

**Files:**
- Modify: `src/deep/discovery.rs`
- Modify: `src/deep/types.rs` (add story stage constants and checker)
- Test: `tests/deep_integration_test.rs`

**Step 1: Add story stage constants and checker**

In `src/deep/types.rs`, add constants near the existing discovery constants:

```rust
// ── Story Chain Thresholds ──────────────────────────────────────────────────

/// Rift Resonance thresholds for each story stage.
pub const STORY_RESONANCE_TREMORS: u32 = 1;
pub const STORY_RESONANCE_CAPTAIN: u32 = 3;
pub const STORY_RESONANCE_FRAGMENT: u32 = 5;
pub const STORY_RESONANCE_ENTRANCE: u32 = 7;

/// Prestige rank gates for each story stage.
pub const STORY_PRESTIGE_TREMORS: u32 = 15;
pub const STORY_PRESTIGE_CAPTAIN: u32 = 17;
pub const STORY_PRESTIGE_FRAGMENT: u32 = 19;
pub const STORY_PRESTIGE_ENTRANCE: u32 = 21;

/// The layer where the Gateway is located.
pub const GATEWAY_LAYER: u32 = 30;
```

Add a method to `impl DeepState`:

```rust
    /// Check and advance story stage based on current rift resonance and prestige rank.
    /// Returns the new stage if it advanced, or None if unchanged.
    pub fn check_story_progression(&mut self, prestige_rank: u32) -> Option<u8> {
        let resonance = self.persistent.rift_resonance;
        let stage = self.persistent.deep_story_stage;

        let new_stage = if stage == 0
            && resonance >= STORY_RESONANCE_TREMORS
            && prestige_rank >= STORY_PRESTIGE_TREMORS
        {
            1
        } else if stage == 1
            && resonance >= STORY_RESONANCE_CAPTAIN
            && prestige_rank >= STORY_PRESTIGE_CAPTAIN
        {
            2
        } else if stage == 2
            && resonance >= STORY_RESONANCE_FRAGMENT
            && prestige_rank >= STORY_PRESTIGE_FRAGMENT
        {
            3
        } else if stage == 3
            && resonance >= STORY_RESONANCE_ENTRANCE
            && prestige_rank >= STORY_PRESTIGE_ENTRANCE
        {
            // Also requires 4 rift fragments
            if self.persistent.rift_fragments >= 4 {
                4
            } else {
                return None;
            }
        } else {
            return None;
        };

        self.persistent.deep_story_stage = new_stage;
        Some(new_stage)
    }

    /// Award a rift fragment if resonance qualifies and no fragment this cycle.
    /// Fragments are awarded at resonance 5, 6, 7, and the 4th comes automatically at stage 4.
    pub fn maybe_award_rift_fragment(&mut self) {
        let resonance = self.persistent.rift_resonance;
        let fragments = self.persistent.rift_fragments;
        // Award fragments at resonance 5, 6, 7 (one per resonance level)
        if resonance >= STORY_RESONANCE_FRAGMENT && fragments < resonance.saturating_sub(STORY_RESONANCE_FRAGMENT - 1).min(4) as u8 {
            self.persistent.rift_fragments = resonance.saturating_sub(STORY_RESONANCE_FRAGMENT - 1).min(4) as u8;
        }
    }
```

**Step 2: Update discovery.rs — add story-chain-based discovery**

In `src/deep/discovery.rs`, add a new public function:

```rust
/// Check if the story chain should advance and optionally discover The Deep.
///
/// Called after rift resonance is incremented (during prestige).
/// Returns the new story stage if it advanced, or None.
pub fn advance_deep_story(deep: &mut DeepState, prestige_rank: u32) -> Option<u8> {
    if deep.persistent.discovered {
        return None;
    }

    // Award fragments based on resonance
    deep.maybe_award_rift_fragment();

    // Check stage progression
    let new_stage = deep.check_story_progression(prestige_rank)?;

    // Stage 4 means the entrance opened — mark as ready for discovery
    // (actual discovery happens when the player presses [D])
    Some(new_stage)
}

/// Complete the discovery via the story chain. Called when the player
/// presses [D] after stage 4 is reached.
pub fn complete_story_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered || deep.persistent.deep_story_stage < 4 {
        return;
    }
    deep.persistent.discovered = true;
    deep.persistent.deep_story_stage = 5;
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.prestige.roster.extend(starters);
    deep.prestige.available_missions =
        super::missions::generate_mission_pool(&deep.persistent, rng);
    deep.prestige.warband_marks = match deep.persistent.guild_rank.0 {
        1 => 50,
        2 => 100,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 50,
    };
}
```

**Step 3: Write tests**

In `tests/deep_integration_test.rs`, add:

```rust
// =========================================================================
// Story Chain Progression
// =========================================================================

#[test]
fn test_story_stage_advances_at_resonance_thresholds() {
    let mut deep = DeepState::new();

    // Stage 0 -> 1 at resonance 1, P15
    deep.persistent.rift_resonance = 1;
    assert_eq!(deep.check_story_progression(15), Some(1));
    assert_eq!(deep.persistent.deep_story_stage, 1);

    // Stage 1 -> 2 at resonance 3, P17
    deep.persistent.rift_resonance = 3;
    assert_eq!(deep.check_story_progression(17), Some(2));
    assert_eq!(deep.persistent.deep_story_stage, 2);

    // Stage 2 -> 3 at resonance 5, P19
    deep.persistent.rift_resonance = 5;
    assert_eq!(deep.check_story_progression(19), Some(3));
    assert_eq!(deep.persistent.deep_story_stage, 3);
}

#[test]
fn test_story_stage_blocked_by_prestige_gate() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 3;
    // Resonance 3 but only P15 — captain requires P17
    deep.persistent.deep_story_stage = 1;
    assert_eq!(deep.check_story_progression(15), None);
    assert_eq!(deep.persistent.deep_story_stage, 1);
}

#[test]
fn test_story_stage_4_requires_fragments() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 7;
    deep.persistent.deep_story_stage = 3;
    deep.persistent.rift_fragments = 3; // need 4
    assert_eq!(deep.check_story_progression(21), None);

    deep.persistent.rift_fragments = 4;
    assert_eq!(deep.check_story_progression(21), Some(4));
}

#[test]
fn test_rift_fragments_awarded_at_resonance_5_6_7() {
    let mut deep = DeepState::new();

    deep.persistent.rift_resonance = 4;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 0, "No fragment before resonance 5");

    deep.persistent.rift_resonance = 5;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 1);

    deep.persistent.rift_resonance = 6;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 2);

    deep.persistent.rift_resonance = 7;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 3);
}

#[test]
fn test_story_discovery_not_triggered_before_stage_4() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 3;
    quest::deep::discovery::complete_story_discovery(&mut deep, &mut rng);
    assert!(!deep.persistent.discovered);
}

#[test]
fn test_story_discovery_completes_at_stage_4() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 4;
    quest::deep::discovery::complete_story_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deep_story_stage, 5);
    assert_eq!(deep.prestige.roster.len(), 3);
}
```

**Step 4: Run tests**

Run: `cargo test --test deep_integration_test`
Expected: All pass.

**Step 5: Commit**

```bash
git add src/deep/types.rs src/deep/discovery.rs tests/deep_integration_test.rs
git commit -m "feat(deep): implement story chain progression with rift resonance gates"
```

---

## Task 4: Wire Story Progression into Prestige Flow

**Files:**
- Modify: `src/input/prestige_input.rs`
- Modify: `src/deep/types.rs` (DeepUiState — add story event fields)

**Step 1: Add story event fields to DeepUiState**

In `src/deep/types.rs`, in the `DeepUiState` struct, add:

```rust
    /// Pending story event to show the player (set during prestige, shown next tick).
    pub pending_story_stage: Option<u8>,
```

**Step 2: Wire advance_deep_story into prestige_input.rs**

In both prestige paths in `src/input/prestige_input.rs`, AFTER the `maybe_increment_rift_resonance` call and BEFORE `deep.on_prestige()`, add:

```rust
                    if let Some(new_stage) = crate::deep::discovery::advance_deep_story(deep, state.prestige_rank) {
                        deep_ui.pending_story_stage = Some(new_stage);
                    }
```

**Step 3: Run tests**

Run: `cargo test --test deep_integration_test`
Expected: All pass.

**Step 4: Commit**

```bash
git add src/deep/types.rs src/input/prestige_input.rs
git commit -m "feat(deep): wire story progression into prestige flow"
```

---

## Task 5: Story Event Modals (UI)

**Files:**
- Modify: `src/ui/deep_scene.rs` (add story modal renderer)
- Modify: `src/input/deep_input.rs` (dismiss story modals)
- Modify: `src/deep/types.rs` (DeepUiState)

**Step 1: Add story modal text constants**

In `src/ui/deep_scene.rs`, add a helper function:

```rust
fn story_modal_content(stage: u8) -> (&'static str, &'static str, &'static [&'static str]) {
    match stage {
        1 => (
            " The Rift Remembers ",
            "Yellow",
            &[
                "The ground shudders beneath the Abyssal Rift.",
                "",
                "Not an earthquake. Not a tremor. Something deeper —",
                "a recognition. The wound in reality shifts, as if",
                "noticing you for the first time.",
                "",
                "\"Every time you return, it opens a little wider.\"",
                "",
                "The feeling passes. But the memory of it does not.",
            ],
        ),
        2 => (
            " The Captain ",
            "Yellow",
            &[
                "A scarred mercenary captain appears at your camp.",
                "Maps spill from worn satchels. Her eyes are old.",
                "",
                "\"I've been tracking the tremors for years. Every",
                "  prestige cycle, the Rift opens a little wider.",
                "  It knows you now.\"",
                "",
                "She spreads a map across the table. It shows depths",
                "below the Rift that no cartographer has charted.",
                "",
                "\"The Rift isn't a wound. It's a door. And something",
                "  on the other side wants it opened.\"",
            ],
        ),
        3 => (
            " The First Fragment ",
            "Cyan",
            &[
                "After the battle, something materializes in the air",
                "where the Rift Behemoth fell. A shard of solidified",
                "void. It hums with a frequency that makes your",
                "teeth ache.",
                "",
                "It wasn't dropped. It was given.",
                "",
                "The captain takes it, holds it to the light.",
                "\"There are more. They appear for those who keep",
                "  coming back. The Rift is testing whether you're",
                "  worth opening for.\"",
                "",
                "Rift Fragment: 1 of 4",
            ],
        ),
        4 => (
            " The Entrance ",
            "Green",
            &[
                "The captain arranges four fragments on the ground.",
                "They snap together — not fitting, but remembering.",
                "",
                "The earth opens.",
                "",
                "A stairway descends into absolute darkness. Cold air",
                "rises from below, carrying the smell of wet stone",
                "and something older. Much older.",
                "",
                "\"I'll need soldiers,\" the captain says.",
                "\"Disposable ones.\"",
                "",
                "Press [D] to descend into The Deep.",
            ],
        ),
        _ => (
            " Story Event ",
            "White",
            &["Something has changed."],
        ),
    }
}
```

**Step 2: Add `render_story_modal()` function**

Follow the same centered-modal pattern as `render_farewell_modal()`. Render the modal when `deep_ui.pending_story_stage.is_some()` and the Deep overlay is NOT open. Show the modal as a centered bordered block with the title and lines of text. Dismiss with Enter or Esc, which clears `pending_story_stage`.

**Step 3: Handle dismiss in input**

In `src/input/mod.rs` or `src/input/deep_input.rs`, add handling: when `deep_ui.pending_story_stage.is_some()`, Enter or Esc clears the field and returns `InputResult::Redraw`.

**Step 4: Run tests**

Run: `cargo test`
Expected: All pass (UI changes are visual-only, no logic tests needed).

**Step 5: Commit**

```bash
git add src/ui/deep_scene.rs src/input/deep_input.rs src/deep/types.rs
git commit -m "feat(deep): add story event modals for Rift Resonance chain"
```

---

## Task 6: Tier-Specific Hub Atmosphere Messages

**Files:**
- Modify: `src/ui/deep_missions.rs` (~lines 341-350)

**Step 1: Replace static atmosphere messages with tier-keyed messages**

Replace the single `atmosphere_messages` array with a function that selects messages based on the frontier layer tier. The function takes `frontier_layer: u32` and returns a `&[&str]` slice.

```rust
fn tier_atmosphere_messages(frontier_layer: u32) -> &'static [&'static str] {
    match LayerTier::from_layer(frontier_layer) {
        LayerTier::Shallows => &[
            "The walls here were carved with purpose. This was no mine.",
            "Your scouts find a collapsed barracks. Decades of dust. Someone lived down here.",
            "The captain traces a finger along a carved warning. She does not translate it.",
            "Tool marks on the walls change from picks to ritual implements.",
            "The tunnels breathe. Your company awaits orders.",
        ],
        LayerTier::Warrens => &[
            "Gareth found a child's doll in the rubble. Stone, but carefully carved.",
            "The archive tablets mention 'the Wellspring' seventeen times.",
            "The Overseer's body twitches even in death. Its purpose outlasted its makers.",
            "Living quarters line these corridors. Families lived here. Children played here.",
            "Distant rumbles echo from below. The Deep stirs.",
        ],
        LayerTier::Hollows => &[
            "The walls pulse with a slow rhythm. It matches your heartbeat.",
            "Your Arcanist says the light here isn't bioluminescence. It's memory.",
            "An Echo walks past the camp. It doesn't see you. It never will.",
            "The spore clouds aren't toxic by nature. They're a defense mechanism.",
            "The stone remembers being shaped. It remembers the hands that shaped it.",
        ],
        LayerTier::SunkenReach => &[
            "The seals glow brighter when your Arcanist approaches. They recognize power.",
            "Water pressure should have crushed these chambers millennia ago. The seals hold more than water.",
            "The Drowned King's throne faces downward. Even in death, it watched what lay below.",
            "These chambers were flooded deliberately. They used water as a barrier.",
            "The rune patterns on the seals match the god items. Someone divine built these.",
        ],
        LayerTier::Abyss => &[
            "Your Scout returned from a 6-hour Recon convinced only twenty minutes had passed.",
            "The whispers have stopped offering power. Now they just say: 'Finally.'",
            "The Void Warden doesn't attack. It simply exists in the way, vast and formless and sad.",
            "Injuries heal wrong down here. Faster, but different.",
            "The Wellspring is trying to communicate. It's been alone for millennia.",
        ],
        LayerTier::Void => &[
            "There is no stone here. Your mercs walk on solidified will.",
            "The Wellspring pulses. It has been waiting longer than your world has existed.",
            "Your Vanguard's wounds close before the Medic reaches them. The power here heals unbidden.",
            "The void is not empty. It is aware.",
            "Each step closer. The Gateway waits at the end of everything.",
        ],
    }
}
```

Update the atmosphere rendering code (~line 351) to call `tier_atmosphere_messages(deep.persistent.frontier_layer())` instead of using the hardcoded array.

Note: The `render_hub` function signature may need to accept `frontier_layer: u32` as an additional parameter. Trace the call chain from `deep_missions.rs` up to determine what context is available. The `DeepPersistent` is accessible — use `persistent.frontier_layer()`.

**Step 2: Run tests**

Run: `cargo test`
Expected: All pass.

**Step 3: Commit**

```bash
git add src/ui/deep_missions.rs
git commit -m "feat(deep): tier-specific hub atmosphere messages for narrative arc"
```

---

## Task 7: Add GatewayExpedition Mission Type

**Files:**
- Modify: `src/deep/types.rs` (MissionType enum)
- Modify: `src/deep/missions.rs` (mission handling)
- Modify: `src/deep/layers.rs` (duration, power threshold)
- Modify: `src/deep/events.rs` (Gateway event templates)
- Test: `tests/deep_integration_test.rs`

**Step 1: Add GatewayExpedition variant**

In `src/deep/types.rs`, add to `MissionType`:

```rust
    /// 24h, Layer 30 only. Opens the Gateway. One-time mission.
    GatewayExpedition,
```

Update `MissionType::display_name()`:

```rust
    MissionType::GatewayExpedition => "Gateway Expedition",
```

Update any `match` on `MissionType` throughout the codebase to include the new variant. Key locations:
- `src/deep/layers.rs` `base_mission_duration_secs()` — return 24h (86400)
- `src/deep/layers.rs` `mission_power_threshold()` — use Layer 29 Breakthrough threshold
- `src/deep/events.rs` `event_trigger_points()` — return 5 trigger points: `&[0.15, 0.35, 0.55, 0.75, 0.90]`
- `src/deep/missions.rs` — any mission pool generation, resolution logic

**Step 2: Add Gateway event templates**

In `src/deep/events.rs`, add 5 static event templates:

```rust
// ── The Gateway (L30) ──────────────────────────────────────────────────────

static GATEWAY_THRESHOLD: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "THE THRESHOLD",
    description: "The void condenses into a corridor. There are walls again — not stone, but light frozen solid. Your mercs can see through them. On the other side, something vast moves.",
    choices: &[
        ChoiceTemplate::safe("Proceed carefully", None, 2 * 3600, 0, 0),
        ChoiceTemplate::safe("Map the light-walls", Some(MercArchetype::Scout), 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static GATEWAY_MEMORY: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "THE MEMORY",
    description: "Echoes of the old civilization's final expedition walk beside your squad. They reached this point. They turned back. Your mercs keep walking.",
    choices: &[
        ChoiceTemplate::safe("Honor their memory", None, 0, 0, 0),
        ChoiceTemplate::risky("Follow where they turned back", 3600, 0.65, 0.15, 80, true),
    ],
    auto_resolve_index: 0,
    risk_success_tag: Some(EventTag::ExploredSidePath),
    risk_failure_tag: None,
};

static GATEWAY_THREE_SEALS: EventTemplate = EventTemplate {
    category: EventCategory::Obstacle,
    title: "THE THREE SEALS",
    description: "Three pedestals. Three recesses shaped like artifacts you recognize — a shield, boots, a belt. The runes match Asprika, Sleipnir, Megingjord. The gateway requires divine keys.",
    choices: &[
        ChoiceTemplate::safe("Study the seals", None, 3600, 0, 0),
        ChoiceTemplate::safe("Channel prestige energy", Some(MercArchetype::Arcanist), 0, 0, 50),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static GATEWAY_WARDENS_PLEA: EventTemplate = EventTemplate {
    category: EventCategory::BossApproach,
    title: "THE WARDEN'S PLEA",
    description: "The Void Warden materializes — not to fight, but to speak. 'Every seal I am was placed by someone who understood what lies beyond. They chose to close it. You are choosing to open it. Be certain.'",
    choices: &[
        ChoiceTemplate::safe("We're certain.", None, 0, 0, 0),
        ChoiceTemplate::risky("Break through by force", 0, 0.70, 0.20, 0, false),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};

static GATEWAY_OPENS: EventTemplate = EventTemplate {
    category: EventCategory::Discovery,
    title: "THE GATE OPENS",
    description: "Your mercs place their hands on the gate. It doesn't open — it dissolves. Beyond it: light. Not sunlight. Older. The Wellspring.",
    choices: &[
        ChoiceTemplate::safe("Step back.", None, 0, 0, 0),
    ],
    auto_resolve_index: 0,
    risk_success_tag: None,
    risk_failure_tag: None,
};
```

Add a static array and function for gateway events:

```rust
static GATEWAY_EVENTS: [&EventTemplate; 5] = [
    &GATEWAY_THRESHOLD,
    &GATEWAY_MEMORY,
    &GATEWAY_THREE_SEALS,
    &GATEWAY_WARDENS_PLEA,
    &GATEWAY_OPENS,
];
```

In `generate_mission_events_with_names()`, add special handling for `GatewayExpedition`: use the 5 gateway events in fixed order (no randomisation, no category diversity check).

**Step 3: Add Gateway resolution logic**

In `src/deep/missions.rs`, in the mission resolution function, when `mission_type == GatewayExpedition` and outcome is success, set `deep.persistent.gateway_opened = true`.

**Step 4: Register in ALL_TEMPLATES**

Add the 5 gateway templates to `ALL_TEMPLATES` in `events.rs` for template lookup.

**Step 5: Write tests**

In `tests/deep_integration_test.rs`:

```rust
// =========================================================================
// Gateway Expedition
// =========================================================================

#[test]
fn test_gateway_expedition_generates_5_events() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let events = quest::deep::events::generate_mission_events(
        MissionType::GatewayExpedition,
        30,
        &[MercArchetype::Vanguard, MercArchetype::Scout, MercArchetype::Arcanist],
        &mut rng,
    );
    assert_eq!(events.len(), 5);
    assert_eq!(events[0].title, "THE THRESHOLD");
    assert_eq!(events[4].title, "THE GATE OPENS");
}

#[test]
fn test_gateway_expedition_events_in_fixed_order() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    let events = quest::deep::events::generate_mission_events(
        MissionType::GatewayExpedition,
        30,
        &[],
        &mut rng,
    );
    let titles: Vec<&str> = events.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, vec![
        "THE THRESHOLD",
        "THE MEMORY",
        "THE THREE SEALS",
        "THE WARDEN'S PLEA",
        "THE GATE OPENS",
    ]);
}
```

**Step 6: Run tests**

Run: `cargo test --test deep_integration_test`
Expected: All pass.

**Step 7: Commit**

```bash
git add src/deep/types.rs src/deep/missions.rs src/deep/layers.rs src/deep/events.rs tests/deep_integration_test.rs
git commit -m "feat(deep): add GatewayExpedition mission type with 5 unique narrative events"
```

---

## Task 8: Add GatewayOpened Achievement

**Files:**
- Modify: `src/achievements/types.rs`
- Modify: `src/achievements/data.rs`
- Test: `tests/deep_integration_test.rs`

**Step 1: Add achievement variant**

In `src/achievements/types.rs`, add to `AchievementId` enum (in The Deep section, after existing Deep achievements):

```rust
    GatewayOpened,            // Opened the Gateway beneath the world
```

**Step 2: Add achievement definition**

In `src/achievements/data.rs`, add the definition following the existing pattern for Deep achievements:

```rust
Achievement {
    id: AchievementId::GatewayOpened,
    name: "The Gate Opens",
    description: "Opened the Gateway beneath the world",
    category: AchievementCategory::Deep,
    hidden: false,
},
```

**Step 3: Wire unlock trigger**

In the appropriate handler (likely `src/achievements/handlers.rs` or wherever Deep mission completion is processed), add a check: when `deep.persistent.gateway_opened` becomes true, unlock `AchievementId::GatewayOpened`.

Alternatively, add a check in `check_milestones()` or create a new handler `on_gateway_opened()`.

**Step 4: Run tests**

Run: `cargo test`
Expected: All pass.

**Step 5: Commit**

```bash
git add src/achievements/types.rs src/achievements/data.rs src/achievements/handlers.rs
git commit -m "feat(deep): add GatewayOpened achievement"
```

---

## Task 9: Gateway Hub State and Post-Completion Display

**Files:**
- Modify: `src/ui/deep_missions.rs` (hub display when gateway is open)

**Step 1: Add gateway status to hub header**

When `deep.persistent.gateway_opened` is true, show a permanent status line in the hub:

```
"The Gateway stands open. The Wellspring waits."
```

This replaces the normal atmosphere message rotation when the gateway is open. Use a distinctive color (e.g., `Color::Rgb(255, 215, 0)` — gold) to make it stand out.

**Step 2: Run tests**

Run: `cargo test`
Expected: All pass.

**Step 3: Commit**

```bash
git add src/ui/deep_missions.rs
git commit -m "feat(deep): show gateway status in hub after completion"
```

---

## Task 10: Update Deep Simulator for New Types

**Files:**
- Modify: `src/bin/deep_simulator.rs`

**Step 1: Add new fields**

Update any `DeepPersistent` construction in the simulator to include the new fields:

```rust
rift_resonance: 0,
deep_story_stage: 5, // simulator assumes discovered
rift_fragments: 4,
gateway_opened: false,
```

Add `GatewayExpedition` handling to any `match` on `MissionType` in the simulator.

**Step 2: Run simulator**

Run: `cargo run --release --bin deep_simulator -- --help`
Expected: Compiles and runs without error.

**Step 3: Commit**

```bash
git add src/bin/deep_simulator.rs
git commit -m "chore(deep): update simulator for narrative fields and GatewayExpedition"
```

---

## Task 11: Final Verification

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, lint, test, build, audit).

**Step 2: Verify backward compatibility**

All new fields use `#[serde(default)]`. Existing save files will load correctly with defaults:
- `rift_resonance: 0`
- `deep_story_stage: 0` (or 5 if already discovered via old system — but the code handles `discovered: true` bypassing the chain)
- `rift_fragments: 0`
- `gateway_opened: false`

**Step 3: Commit any fixes**

If `make check` reveals issues, fix and commit.

---

## Team Assignment

This plan maps to the requested team of 12. Each task can be assigned to a team member based on their role:

| Task | Assignee Role | Description |
|------|--------------|-------------|
| 1 | Dev 1 | Type additions + serde tests |
| 2 | Dev 1 | Rift Resonance prestige hook |
| 3 | Dev 2 | Story chain progression logic |
| 4 | Dev 2 | Wire into prestige flow |
| 5 | Story Writer 1 + Dev 1 | Story modal text + UI rendering |
| 6 | Story Writer 2 + Story Writer 3 | Tier atmosphere messages (18 messages across 6 tiers) |
| 7 | Game Designer 1 + Game Designer 2 + Dev 2 | Gateway mission type + 5 event templates |
| 8 | Dev 1 | Achievement addition |
| 9 | Dev 2 | Hub gateway state display |
| 10 | Eng Mgr | Simulator update |
| 11 | QA 1 + QA 2 + Sys Arch | Final verification |

**Parallelism opportunities:**
- Tasks 1-2 (types + prestige hook) can run in parallel with Task 6 (atmosphere messages — pure content)
- Tasks 3-4 (story logic) depend on Task 1
- Task 5 (modals) depends on Tasks 3-4
- Task 7 (gateway mission) depends on Task 1
- Tasks 8-9 depend on Task 7
- Tasks 10-11 are final and sequential

## 2026-02-26-deep-discovery-redesign-plan.md

# Deep Discovery Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the 30-prestige Rift Resonance story chain with a single trigger: killing The Endless (Zone 11 boss) at P15+.

**Architecture:** Remove all story chain machinery (rift resonance, 10 stages, story modals). Add a discovery check in `tick_stages.rs` when `BossDefeatResult::ExpanseCycle` fires. Rewrite `discovery.rs` to a simple `complete_discovery()`. Remove `rift_hint`/`rift_resonance` parameter threading from all UI functions.

**Tech Stack:** Rust, Ratatui

---

### Task 1: Rewrite `discovery.rs` — remove story chain, simplify to boss-trigger discovery

**Files:**
- Modify: `src/deep/discovery.rs`

**Step 1: Rewrite discovery.rs**

Remove `advance_deep_story()`. Rewrite `complete_story_discovery()` to `complete_discovery()` — remove the `deep_story_stage` guard (just check `discovered`). Keep `queue_first_orders()` unchanged.

```rust
use super::mercenaries::generate_starter_roster;
use super::types::{DeepState, MercStatus, Mission, MissionStatus, MissionType};
use chrono::Utc;
use rand::Rng;

/// Complete The Deep discovery. Called when the player kills The Endless
/// (Zone 11 boss) for the first time at P15+.
pub fn complete_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered {
        return;
    }
    deep.persistent.discovered = true;
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.prestige.roster.extend(starters);
    deep.prestige.available_missions =
        super::missions::generate_mission_pool(&deep.persistent, rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());
    deep.prestige.warband_marks = match deep.persistent.guild_rank.0 {
        1 => 50,
        2 => 100,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 50,
    };
    queue_first_orders(deep);
}

// queue_first_orders stays exactly the same
```

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: Compile errors in files that reference old functions (mod.rs, prestige_input.rs, etc.) — that's expected, we'll fix those next.

**Step 3: Commit**

```
git add src/deep/discovery.rs
git commit -m "refactor(deep): rewrite discovery.rs — remove story chain, simplify to complete_discovery()"
```

---

### Task 2: Remove story chain constants and fields from `types.rs`

**Files:**
- Modify: `src/deep/types.rs`

**Step 1: Remove story chain constants**

Delete lines 15-26 (the `STORY_RESONANCE_THRESHOLDS`, `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED` constants).

**Step 2: Remove `rift_resonance` and `deep_story_stage` fields from `DeepPersistent`**

In the struct definition (~line 696-701), remove both fields. In `DeepPersistent::new()` (~line 730-731), remove the initializers.

**Step 3: Remove `maybe_increment_rift_resonance()` and `check_story_progression()` methods**

Delete the `maybe_increment_rift_resonance()` method (~lines 917-924) and `check_story_progression()` method (~lines 926-948) from `impl DeepState`.

**Step 4: Remove `pending_story_stage` from `DeepUiState`**

Remove the field (~line 1077) and its initializer (~line 1099).

**Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -30`
Expected: More compile errors from consumers — expected.

**Step 6: Commit**

```
git add src/deep/types.rs
git commit -m "refactor(deep): remove rift resonance, story stage fields, and story chain methods"
```

---

### Task 3: Update `mod.rs` re-exports

**Files:**
- Modify: `src/deep/mod.rs`

**Step 1: Update re-exports**

Remove from the `pub use types` block: `STORY_RESONANCE_THRESHOLDS`, `STORY_STAGE_DISCOVERED`, `STORY_STAGE_ENTRANCE`.

Replace the discovery re-export line:
```rust
// Old:
pub use discovery::{advance_deep_story, complete_story_discovery};
// New:
pub use discovery::complete_discovery;
```

**Step 2: Commit**

```
git add src/deep/mod.rs
git commit -m "refactor(deep): update mod.rs re-exports for simplified discovery"
```

---

### Task 4: Remove story chain code from prestige input

**Files:**
- Modify: `src/input/prestige_input.rs`

**Step 1: Remove rift resonance and story chain calls**

In `handle_vault_selection()` (~lines 71-80): Delete the block that calls `maybe_increment_rift_resonance()` and `advance_deep_story()` / sets `pending_story_stage`.

In `handle_prestige_confirm()` (~lines 153-162): Same removal.

Remove the `deep_ui` parameter from both functions if it's only used for `pending_story_stage` — check first whether `farewell_mercs` still needs it (it does). Keep `deep_ui` param, just remove the story chain lines.

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 3: Commit**

```
git add src/input/prestige_input.rs
git commit -m "refactor(deep): remove rift resonance and story chain from prestige input"
```

---

### Task 5: Remove story modal UI and pending_story_stage input handling

**Files:**
- Modify: `src/ui/deep_scene.rs`
- Modify: `src/main_helpers/overlay.rs`
- Modify: `src/input/mod.rs`

**Step 1: Remove story modal functions from `deep_scene.rs`**

Delete `story_modal_content()` (~lines 565-706) and `render_story_modal()` (~lines 708-756).

**Step 2: Remove story modal rendering from `overlay.rs`**

Delete the block at ~lines 263-266:
```rust
if let Some(stage) = deep_ui.pending_story_stage {
    ui::deep_scene::render_story_modal(frame, area, stage);
}
```

**Step 3: Remove story modal input handling from `input/mod.rs`**

Delete the block at ~lines 77-83:
```rust
// 0.4. Deep story event modal (Enter or Esc dismisses)
if deep_ui.pending_story_stage.is_some() {
    ...
}
```

**Step 4: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 5: Commit**

```
git add src/ui/deep_scene.rs src/main_helpers/overlay.rs src/input/mod.rs
git commit -m "refactor(deep): remove story modal UI and input handling"
```

---

### Task 6: Remove `rift_hint` and `rift_resonance` parameter threading from UI

**Files:**
- Modify: `src/ui/prestige_confirm.rs`
- Modify: `src/ui/stats_prestige.rs`
- Modify: `src/ui/stats_panel.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/main_helpers/overlay.rs`

**Step 1: Simplify `draw_prestige_confirm()`**

Remove `rift_hint` parameter. Remove the `if rift_hint { ... }` block (~lines 101-115). Set `base_height` to always `18`.

**Step 2: Simplify `draw_prestige_panel()` in `stats_prestige.rs`**

Remove `rift_hint` and `rift_resonance` parameters. Remove the `if rift_hint { ... }` conditional (~lines 214-232) — always use the `unlock_hint` path.

**Step 3: Remove `rift_hint`/`rift_resonance` from `draw_stats_panel()` in `stats_panel.rs`**

Remove both parameters from the function signature (~lines 69-70) and the call to `draw_prestige_panel()` (~lines 101-102).

**Step 4: Remove from `draw_xl_l_layout()` and `draw_game_layout()` in `ui/mod.rs`**

Remove both parameters from `draw_xl_l_layout()` signature (~lines 456-457) and its call site (~lines 416-417). Remove the `rift_hint`/`rift_resonance` computation (~lines 396-399) from `draw_game_layout()`.

**Step 5: Remove from `draw_prestige_confirm` call in `overlay.rs`**

In `draw_game_overlays()` (~lines 108-111): Remove the `rift_hint` local and pass only `(frame, state, ctx)`.

**Step 6: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 7: Commit**

```
git add src/ui/prestige_confirm.rs src/ui/stats_prestige.rs src/ui/stats_panel.rs src/ui/mod.rs src/main_helpers/overlay.rs
git commit -m "refactor(deep): remove rift_hint/rift_resonance UI parameter threading"
```

---

### Task 7: Add discovery trigger on ExpanseCycle in tick_stages.rs

**Files:**
- Modify: `src/core/tick_stages.rs`
- Modify: `src/core/tick_types.rs`

**Step 1: Add `TickEvent::DeepDiscovered` variant**

In `tick_types.rs`, add to the Discovery section (after `StormglassDiscovered`):
```rust
/// The Deep was discovered (first Endless kill at P15+).
DeepDiscovered,
```

**Step 2: Add `deep_discovered` to `TickResult`**

Not needed — `deep_changed` already exists and covers this.

**Step 3: Trigger discovery on `ExpanseCycle`**

In `tick_stages.rs`, in the `process_combat_events()` function, find the `BossDefeatResult::ExpanseCycle` match arm (~line 558). After the existing message formatting, add the discovery check:

```rust
BossDefeatResult::ExpanseCycle => {
    // ... existing message code ...

    // Check for Deep discovery on first Endless kill
    if !deep.persistent.discovered
        && state.prestige_rank >= crate::deep::DEEP_MIN_PRESTIGE_RANK
    {
        crate::deep::complete_discovery(deep, rng);
        result.events.push(TickEvent::DeepDiscovered);
        result.deep_changed = true;
        achievements.on_deep_discovered(Some(&state.character_name));
        if !debug_mode {
            result.achievements_changed = true;
        }
    }
}
```

This requires adding `deep: &mut DeepState` and `debug_mode: bool` parameters to `process_combat_events()`. Update the function signature and its call site in `tick.rs`.

**Step 4: Handle `TickEvent::DeepDiscovered` in `tick_events.rs`**

Add a match arm in the tick event processing that sets `deep_discovered` flag (same pattern as `haven_discovered`/`soulforge_discovered`). In `main.rs`, map this flag to `GameOverlay::DeepDiscovery`.

Check how `HavenDiscovered` and `SoulforgeDiscovered` are handled in `tick_events.rs` and `main.rs` first — follow the exact same pattern.

**Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 6: Commit**

```
git add src/core/tick_stages.rs src/core/tick_types.rs src/tick_events.rs src/core/tick.rs
git commit -m "feat(deep): trigger Deep discovery on first Endless kill at P15+"
```

---

### Task 8: Update debug menu

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Update debug menu discovery shortcut**

In `trigger_deep_discovery()` (~line 626-631): Remove the `deep_story_stage` assignment. Change `complete_story_discovery` to `complete_discovery`:

```rust
fn trigger_deep_discovery(deep: &mut crate::deep::DeepState, _prestige_rank: u32) -> &'static str {
    let mut rng = rand::rng();
    crate::deep::complete_discovery(deep, &mut rng);
    "The Deep discovered!"
}
```

**Step 2: Commit**

```
git add src/utils/debug_menu.rs
git commit -m "refactor(deep): update debug menu to use simplified discovery"
```

---

### Task 9: Update tests

**Files:**
- Modify: `tests/deep_tutorial_test.rs`
- Modify: `tests/deep_types_coverage_test.rs`
- Modify: `tests/deep_prestige_persistence_test.rs`
- Modify: `tests/deep_integration_test.rs`

**Step 1: Rewrite story chain tests in `deep_tutorial_test.rs`**

The tests starting around line 880 (`test_deep_story_chain_full_progression`, `test_deep_story_final_stage_requires_resonance_and_p15`, `test_deep_discovery_only_once_via_game_tick`, `test_rift_resonance_only_increments_in_expanse`) test the removed story chain. Replace them with a single test for the new trigger:

```rust
#[test]
fn test_deep_discovery_on_endless_kill() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();

    // Discovery requires !discovered
    assert!(!deep.persistent.discovered);

    quest::deep::complete_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.prestige.roster.len(), 3);
    assert_eq!(deep.prestige.warband_marks, 50);
    assert!(!deep.prestige.active_missions.is_empty()); // First Orders queued
}

#[test]
fn test_deep_discovery_is_idempotent() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();

    quest::deep::complete_discovery(&mut deep, &mut rng);
    let roster_count = deep.prestige.roster.len();

    // Calling again should not double-init
    quest::deep::complete_discovery(&mut deep, &mut rng);
    assert_eq!(deep.prestige.roster.len(), roster_count);
}
```

**Step 2: Fix any tests referencing removed fields**

Search test files for `rift_resonance`, `deep_story_stage`, `advance_deep_story`, `complete_story_discovery`, `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED`, `STORY_RESONANCE_THRESHOLDS`, `pending_story_stage`, `maybe_increment_rift_resonance`, `check_story_progression`. Remove or update each reference.

**Step 3: Run the full test suite**

Run: `cargo test 2>&1 | tail -20`
Expected: All tests pass.

**Step 4: Commit**

```
git add tests/
git commit -m "test(deep): update tests for boss-trigger discovery, remove story chain tests"
```

---

### Task 10: Update documentation

**Files:**
- Modify: `src/deep/CLAUDE.md`
- Modify: `CLAUDE.md`

**Step 1: Update `src/deep/CLAUDE.md`**

Update the Discovery section to reflect boss-trigger instead of story chain. Remove references to Rift Resonance, story stages, and per-tick random roll. Update the constants table (remove story chain constants). Update integration points (discovery now happens in tick_stages.rs on ExpanseCycle, not in prestige_input.rs).

**Step 2: Update root `CLAUDE.md`**

Update The Deep Module description if it mentions story chain or Rift Resonance.

**Step 3: Commit**

```
git add src/deep/CLAUDE.md CLAUDE.md
git commit -m "docs: update CLAUDE.md files for boss-trigger Deep discovery"
```

---

### Task 11: Final verification

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, test, build, audit).

**Step 2: Commit any formatting fixes**

Run: `make fmt` if needed, then commit.

## 2026-03-02-deep-panel-plan.md

# Unified Deep Panel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Power Cores panel with a unified "The Deep" panel that shows guild rank, missions, crew status, and compact core readiness badges in 8 rows.

**Architecture:** Rename/rewrite `draw_power_cores_panel()` in `src/ui/stats_prestige.rs` to `draw_deep_panel()`. The function signature gains access to `DeepState` (already passed). The layout in `stats_panel.rs` changes the visibility condition from "any unlocked core" to "Deep discovered". Height stays at 8 rows.

**Tech Stack:** Ratatui (Span, Line, Paragraph, Block), chrono (Utc::now for mission ETA), existing Deep types.

---

### Task 1: Add helper function for next mission ETA

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

Add at the bottom of the existing `#[cfg(test)] mod tests` block in `src/ui/stats_prestige.rs`:

```rust
#[test]
fn test_next_mission_eta_no_missions() {
    let prestige = crate::deep::DeepPrestige::default();
    assert_eq!(next_mission_eta_secs(&prestige), None);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_next_mission_eta_no_missions`
Expected: FAIL — `next_mission_eta_secs` not found

**Step 3: Write minimal implementation**

Add above the `#[cfg(test)]` block in `src/ui/stats_prestige.rs`:

```rust
/// Returns seconds until the next active mission completes, or None if no active missions.
fn next_mission_eta_secs(prestige: &crate::deep::DeepPrestige) -> Option<i64> {
    let now = chrono::Utc::now();
    prestige
        .active_missions
        .iter()
        .filter(|m| matches!(m.status, crate::deep::MissionStatus::Active | crate::deep::MissionStatus::EventPending))
        .map(|m| (m.ends_at - now).num_seconds().max(0))
        .min()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_next_mission_eta_no_missions`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add next_mission_eta_secs helper for Deep panel"
```

---

### Task 2: Add helper function for pending event count

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_pending_event_count_no_events() {
    let prestige = crate::deep::DeepPrestige::default();
    assert_eq!(pending_event_count(&prestige), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_pending_event_count_no_events`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
/// Count of active missions with pending events needing player response.
fn pending_event_count(prestige: &crate::deep::DeepPrestige) -> usize {
    prestige
        .active_missions
        .iter()
        .filter(|m| m.has_pending_event())
        .count()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_pending_event_count_no_events`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add pending_event_count helper for Deep panel"
```

---

### Task 3: Add helper for core summary data

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_core_summary_no_cores() {
    let achievements = Achievements::default();
    let deep = crate::deep::DeepState::default();
    let summary = core_summary(&achievements, &deep);
    assert_eq!(summary.ready_count, 0);
    assert_eq!(summary.ready_pr, 0);
    assert_eq!(summary.unlocked_count, 0);
    assert_eq!(summary.total_pr_per_day, 0);
    assert!(summary.next_ready_secs.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_core_summary_no_cores`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
struct CoreSummary {
    ready_count: usize,
    ready_pr: u32,
    unlocked_count: usize,
    total_pr_per_day: u32,
    next_ready_secs: Option<i64>,
    /// Per-core status: (is_unlocked, is_ready, time_remaining_secs, required_layer, pr_per_day)
    cores: Vec<CoreBadge>,
}

struct CoreBadge {
    unlocked: bool,
    ready: bool,
    remaining_secs: i64,
    required_layer: u32,
}

fn core_summary(
    achievements: &crate::achievements::Achievements,
    deep: &crate::deep::DeepState,
) -> CoreSummary {
    use crate::power_cores::types::{fill_duration_secs, fill_ratio, ALL_POWER_CORES};
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut summary = CoreSummary {
        ready_count: 0,
        ready_pr: 0,
        unlocked_count: 0,
        total_pr_per_day: 0,
        next_ready_secs: None,
        cores: Vec::new(),
    };

    for core in ALL_POWER_CORES {
        let is_unlocked = achievements.is_unlocked(core.achievement_id);

        if is_unlocked {
            summary.unlocked_count += 1;
            summary.total_pr_per_day += core.pr_per_day;

            let fill_secs = fill_duration_secs(core.pr_per_day);
            let last_granted = deep
                .persistent
                .power_core_last_granted
                .get(&core.achievement_id)
                .copied()
                .unwrap_or(0);
            let elapsed = (now - last_granted).max(0);
            let ratio = fill_ratio(elapsed, fill_secs);
            let remaining = (fill_secs - elapsed).max(0);

            if ratio >= 1.0 {
                summary.ready_count += 1;
                summary.ready_pr += core.pr_per_day;
                summary.cores.push(CoreBadge {
                    unlocked: true,
                    ready: true,
                    remaining_secs: 0,
                    required_layer: core.required_layer,
                });
            } else {
                if let Some(current_next) = summary.next_ready_secs {
                    if remaining < current_next {
                        summary.next_ready_secs = Some(remaining);
                    }
                } else {
                    summary.next_ready_secs = Some(remaining);
                }
                summary.cores.push(CoreBadge {
                    unlocked: true,
                    ready: false,
                    remaining_secs: remaining,
                    required_layer: core.required_layer,
                });
            }
        } else {
            summary.cores.push(CoreBadge {
                unlocked: false,
                ready: false,
                remaining_secs: 0,
                required_layer: core.required_layer,
            });
        }
    }

    summary
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_core_summary_no_cores`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add core_summary helper for Deep panel"
```

---

### Task 4: Rewrite draw_power_cores_panel to draw_deep_panel

**Files:**
- Modify: `src/ui/stats_prestige.rs` — replace `draw_power_cores_panel()` with `draw_deep_panel()`

**Step 1: Rename and rewrite the function**

Replace the entire `draw_power_cores_panel` function (lines 547-656 of `src/ui/stats_prestige.rs`) with `draw_deep_panel`. The new function renders 6 content rows:

```rust
/// Draws the unified Deep panel: guild rank, missions, crew, and power core status.
///
/// Shows when The Deep is discovered. 8 rows total (6 content + 2 border).
pub(super) fn draw_deep_panel(
    frame: &mut Frame,
    area: Rect,
    achievements: &crate::achievements::Achievements,
    deep: &DeepState,
) {
    const AMBER: Color = Color::Rgb(220, 180, 60);
    const CORE_AMBER: Color = Color::Rgb(255, 165, 0);

    if !deep.persistent.discovered {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" The Deep ")
        .border_style(Style::default().fg(super::themed_border_color(CORE_AMBER)));
    let inner = super::render_themed_block(frame, area, block, CORE_AMBER, super::BorderFxContext);

    let mut lines: Vec<Line> = Vec::new();
    let width = inner.width as usize;

    // Row 1: Guild rank + Warband Marks
    {
        let rank_name = deep.persistent.guild_rank.display_name();
        let marks = deep.prestige.warband_marks;
        let marks_str = format!("\u{25c6} {} Warband Marks", marks);
        let rank_part = format!("\u{2b21} {}", rank_name);
        let padding = width.saturating_sub(rank_part.len() + marks_str.len());

        lines.push(Line::from(vec![
            Span::styled("\u{2b21} ", Style::default().fg(Color::White)),
            Span::styled(rank_name.to_string(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(padding)),
            Span::styled(
                format!("\u{25c6} ", ),
                Style::default().fg(AMBER),
            ),
            Span::styled(
                format!("{} Warband Marks", marks),
                Style::default().fg(AMBER),
            ),
        ]));
    }

    // Row 2: Missions + Next completion timer
    {
        let active = deep.prestige.active_mission_count();
        let max_concurrent = crate::deep::effective_concurrent_missions(
            deep.persistent.guild_rank,
            deep.persistent.deepest_layer_reached,
        );
        let mission_str = format!("Missions {}/{}", active, max_concurrent);

        let eta = next_mission_eta_secs(&deep.prestige);
        let eta_str = match eta {
            Some(secs) => format!("\u{25f7} Next: ~{}", format_eta(secs as u64)),
            None => "\u{25f7} idle".to_string(),
        };
        let eta_color = match eta {
            Some(secs) if secs < 900 => Color::Yellow,
            Some(_) => Color::DarkGray,
            None => Color::DarkGray,
        };

        let padding = width.saturating_sub(mission_str.len() + eta_str.len());

        lines.push(Line::from(vec![
            Span::styled(mission_str, Style::default().fg(Color::Cyan)),
            Span::raw(" ".repeat(padding)),
            Span::styled(eta_str, Style::default().fg(eta_color)),
        ]));
    }

    // Row 3: Crew glyphs + Frontier + Events
    {
        let mut crew_spans: Vec<Span> = Vec::new();
        let mut available: Vec<&crate::deep::Mercenary> = Vec::new();
        let mut on_mission: Vec<&crate::deep::Mercenary> = Vec::new();
        let mut injured: Vec<&crate::deep::Mercenary> = Vec::new();

        for merc in &deep.prestige.roster {
            match merc.status {
                crate::deep::MercStatus::Available => available.push(merc),
                crate::deep::MercStatus::OnMission(_) => on_mission.push(merc),
                crate::deep::MercStatus::Injured { .. } => injured.push(merc),
                crate::deep::MercStatus::Lost => {} // skip
            }
        }

        // Available mercs: ♦ (green)
        if !available.is_empty() {
            crew_spans.push(Span::styled(
                "\u{2666}".repeat(available.len()),
                Style::default().fg(Color::Green),
            ));
        }
        // Space between groups
        if !available.is_empty() && (!on_mission.is_empty() || !injured.is_empty()) {
            crew_spans.push(Span::raw(" "));
        }
        // On mission: ♢ (cyan)
        if !on_mission.is_empty() {
            crew_spans.push(Span::styled(
                "\u{2662}".repeat(on_mission.len()),
                Style::default().fg(Color::Cyan),
            ));
        }
        if !on_mission.is_empty() && !injured.is_empty() {
            crew_spans.push(Span::raw(" "));
        }
        // Injured: ✝ (red)
        if !injured.is_empty() {
            crew_spans.push(Span::styled(
                "\u{271d}".repeat(injured.len()),
                Style::default().fg(Color::Red),
            ));
        }

        let crew_width: usize = available.len() + on_mission.len() + injured.len()
            + if !available.is_empty() && (!on_mission.is_empty() || !injured.is_empty()) { 1 } else { 0 }
            + if !on_mission.is_empty() && !injured.is_empty() { 1 } else { 0 };

        // Right side: Frontier + events
        let frontier = deep.persistent.frontier_layer();
        let events = pending_event_count(&deep.prestige);
        let frontier_str = format!("Frontier L{}", frontier);
        let event_str = if events > 0 {
            format!("  \u{26a1}{}", events)
        } else {
            String::new()
        };
        let right_str_len = frontier_str.len() + event_str.len();

        let padding = width.saturating_sub(crew_width + right_str_len);
        crew_spans.push(Span::raw(" ".repeat(padding)));
        crew_spans.push(Span::styled(
            frontier_str,
            Style::default().fg(Color::Rgb(120, 140, 170)),
        ));
        if events > 0 {
            crew_spans.push(Span::styled(
                event_str,
                Style::default().fg(Color::Yellow),
            ));
        }

        lines.push(Line::from(crew_spans));
    }

    // Row 4: Separator
    {
        let sep = "\u{2500}".repeat(width);
        lines.push(Line::from(Span::styled(
            sep,
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Rows 5-6: Core summary + badges
    let summary = core_summary(achievements, deep);
    {
        // Row 5: "Cores: N ✓ Ready (+X PR)  ·  Next: Xh Ym" or "Cores: locked ..."
        let mut spans: Vec<Span> = Vec::new();

        if summary.unlocked_count == 0 {
            let left = "Cores: locked";
            let right = "First core at L3";
            let padding = width.saturating_sub(left.len() + right.len());
            spans.push(Span::styled(left.to_string(), Style::default().fg(Color::DarkGray)));
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(right.to_string(), Style::default().fg(Color::DarkGray)));
        } else if summary.ready_count > 0 && summary.next_ready_secs.is_none() {
            // All unlocked cores are ready
            let left = format!(
                "Cores: {} \u{2713} Ready (+{} PR)",
                summary.ready_count, summary.ready_pr
            );
            let right = "All ready!";
            let padding = width.saturating_sub(left.len() + 4 + right.len());
            spans.push(Span::styled("Cores: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("{} \u{2713} Ready", summary.ready_count),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::styled(
                format!(" (+{} PR)", summary.ready_pr),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(" ".repeat(padding.saturating_sub("Cores: ".len()))));
            spans.push(Span::styled(
                "All ready!".to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            let ready_part = if summary.ready_count > 0 {
                format!(
                    "{} \u{2713} Ready (+{} PR)",
                    summary.ready_count, summary.ready_pr
                )
            } else {
                "0 \u{2713} Ready".to_string()
            };
            let next_part = match summary.next_ready_secs {
                Some(secs) => format!("Next: {}", format_eta(secs as u64)),
                None => String::new(),
            };
            let left_len = "Cores: ".len() + ready_part.len();
            let right_len = next_part.len();
            let padding = width.saturating_sub(left_len + 5 + right_len);

            spans.push(Span::styled("Cores: ", Style::default().fg(Color::DarkGray)));
            if summary.ready_count > 0 {
                spans.push(Span::styled(
                    ready_part,
                    Style::default().fg(Color::Green),
                ));
            } else {
                spans.push(Span::styled(
                    ready_part,
                    Style::default().fg(Color::DarkGray),
                ));
            }
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(
                format!("  \u{00b7}  {}", next_part),
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines.push(Line::from(spans));
    }

    {
        // Row 6: Per-core badges + PR/day
        let mut spans: Vec<Span> = Vec::new();

        for (i, badge) in summary.cores.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            if badge.unlocked {
                spans.push(Span::styled(
                    "\u{2742}",
                    Style::default().fg(CORE_AMBER),
                ));
                if badge.ready {
                    spans.push(Span::styled(
                        "\u{2713}",
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    let time = format_core_time_short(badge.remaining_secs);
                    spans.push(Span::styled(
                        time,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            } else {
                spans.push(Span::styled(
                    format!("\u{25c7}L{}", badge.required_layer),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Right-align PR/day
        let pr_str = format!("+{} PR/day", summary.total_pr_per_day);
        // Calculate current badge width for padding
        let badge_text: String = summary.cores.iter().enumerate().map(|(i, b)| {
            let prefix = if i > 0 { " " } else { "" };
            if b.unlocked {
                if b.ready {
                    format!("{}\u{2742}\u{2713}", prefix)
                } else {
                    format!("{}\u{2742}{}", prefix, format_core_time_short(b.remaining_secs))
                }
            } else {
                format!("{}\u{25c7}L{}", prefix, b.required_layer)
            }
        }).collect();
        let badge_width = badge_text.chars().count();
        let padding = width.saturating_sub(badge_width + pr_str.len());

        if summary.total_pr_per_day > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(pr_str, Style::default().fg(CORE_AMBER)));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

/// Format seconds into short form for core badges: "1h", "2h", "45m", "11h"
fn format_core_time_short(secs: i64) -> String {
    let secs = secs.max(0) as u64;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", minutes.max(1))
    }
}
```

**Step 2: Run clippy and fix any issues**

Run: `cargo clippy --all-targets -- -D warnings`

**Step 3: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: replace Power Cores panel with unified Deep panel"
```

---

### Task 5: Update stats_panel.rs to use draw_deep_panel

**Files:**
- Modify: `src/ui/stats_panel.rs`

**Step 1: Update the import**

In `src/ui/stats_panel.rs` line 7, change the import:
```rust
// Before:
use super::stats_prestige::{
    draw_fishing_panel, draw_power_cores_panel, draw_prestige_info, format_eta,
};

// After:
use super::stats_prestige::{
    draw_deep_panel, draw_fishing_panel, draw_prestige_info, format_eta,
};
```

**Step 2: Update the visibility condition**

Replace the `unlocked_cores` / `power_cores_height` logic (lines 82-87) with:

```rust
// Before:
let unlocked_cores = crate::power_cores::get_unlocked_cores(achievements).len();
let power_cores_height = if unlocked_cores > 0 {
    crate::power_cores::ALL_POWER_CORES.len() as u16 + 2
} else {
    0
};

// After:
let deep_panel_height: u16 = if deep.persistent.discovered { 8 } else { 0 };
```

**Step 3: Update the constraint**

```rust
// Before:
if power_cores_height > 0 {
    constraints.push(Constraint::Length(power_cores_height));
}

// After:
if deep_panel_height > 0 {
    constraints.push(Constraint::Length(deep_panel_height));
}
```

**Step 4: Update the render call**

```rust
// Before:
if power_cores_height > 0 {
    draw_power_cores_panel(frame, chunks[idx], achievements, deep);
    idx += 1;
}

// After:
if deep_panel_height > 0 {
    draw_deep_panel(frame, chunks[idx], achievements, deep);
    idx += 1;
}
```

**Step 5: Run the full check**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build)

**Step 6: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat: wire draw_deep_panel into stats panel layout"
```

---

### Task 6: Clean up old draw_power_cores_panel references

**Files:**
- Modify: `src/ui/stats_prestige.rs` — remove old function if not already replaced
- Modify: `src/ui/mod.rs` — check for any remaining references

**Step 1: Search for remaining references**

Run: `cargo build 2>&1 | grep -i "power_cores_panel\|unused"` to check for dead code warnings.

**Step 2: Remove any dead code**

Remove `draw_power_cores_panel` if it still exists alongside `draw_deep_panel`. Remove any unused imports.

**Step 3: Run full checks**

Run: `make check`
Expected: All checks pass with no warnings

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove dead draw_power_cores_panel code"
```

---

### Task 7: Visual verification and edge case testing

**Step 1: Run the game and verify all states**

Run: `cargo run`

Verify:
- Deep not discovered → panel is hidden
- Deep discovered, no cores → "Cores: locked" row with L3 target
- Cores filling → time remaining shown in badges
- Cores ready → green ✓ badges
- All cores ready → "All ready!" in green bold
- Crew glyphs reflect actual roster state
- Mission count and ETA update correctly
- Events badge shows when missions have pending events

**Step 2: Use debug menu to test different states**

Test with different guild ranks, roster sizes, mission counts, and core unlock levels.

**Step 3: Final commit if any adjustments needed**

```bash
git add -A
git commit -m "fix: adjust Deep panel rendering for edge cases"
```

## deep-implementation-plan.md

# The Deep — Implementation Plan

## Overview

This plan describes the phased implementation of The Deep Mercenary Expedition System for Quest. The Deep is a P15+ endgame feature where players recruit mercenaries and send squads on real-time missions (2-24 hours) that push deeper into an underground structure. It introduces wall-clock time progression — a fundamentally new pattern for Quest.

The plan is organized into 6 phases with clear dependencies, a complete file inventory, risk assessment, minimum viable feature definition, and testing strategy.

---

## Phase 1: Core Types and Data Model

**Blocks everything.** All subsequent phases depend on the type definitions established here.

**Tasks:** #1 (design) -> #8 (implement)

### 1.1 New Files

| File | Contents |
|------|----------|
| `src/deep/mod.rs` | Public re-exports (follows Haven/Enhancement pattern) |
| `src/deep/types.rs` | All core data structures (see below) |
| `src/deep/CLAUDE.md` | Module documentation (follows existing module CLAUDE.md pattern) |

### 1.2 Core Types to Define (`src/deep/types.rs`)

```
// ── Mercenaries ──
MercenaryArchetype       enum (Vanguard, Scout, Arcanist, Medic, Saboteur)
MercenaryStats           struct { power: u32, resilience: u32, expertise: u32 }
MercenaryStatus          enum (Available, OnMission, Injured { recover_at: i64 }, Lost)
Mercenary                struct { id, name, archetype, stats, level, xp, status }

// ── Layers ──
LayerTier                enum (Shallows, Warrens, Hollows, SunkenReach, Abyss, Void)
InfrastructureType       enum (Outpost, SupplyCache, Watchtower, Bridge)
LayerState               struct { layer_num, cleared, familiarity, infrastructure: [Option<InfrastructureType>; 2] }

// ── Missions ──
MissionType              enum (SupplyRun, Recon, Expedition, Breakthrough, Construction)
MissionStatus            enum (Active, PendingEvent, Completed, Failed)
MissionOutcome           enum (FullSuccess, PartialSuccess, Failure)
SquadSlot                struct { mercenary_id: u64 }
EventChoice              struct { label, archetype_bonus, outcome effects }
CheckInEvent             struct { event_id, description, choices, auto_resolve_choice, triggered_at, auto_resolve_at }
PendingEvent             struct { event: CheckInEvent, fired_at_progress: f64 }
Mission                  struct { id, mission_type, layer, squad, started_at: i64, duration_seconds, events, pending_event, status, outcome }

// ── Guild ──
GuildRank                enum/struct (Freelancers..Legion) with max_roster, concurrent_missions, rank requirements

// ── Economy ──
WarbandMarks             u64 (alias or newtype)

// ── Top-Level State ──
DeepState                struct { discovered, guild_rank, marks, roster: Vec<Mercenary>, layers: Vec<LayerState>,
                                  active_missions: Vec<Mission>, completed_missions: Vec<Mission>,
                                  frontier_layer, next_merc_id, recruit_pool, recruit_pool_refreshed_at,
                                  free_daily_used_at, last_tick_time }

// ── Constants ──
DEEP_MIN_PRESTIGE_RANK   = 15
DEEP_DISCOVERY_BASE_CHANCE, DEEP_DISCOVERY_RANK_BONUS (same pattern as Soulforge)
MAX_LAYERS               = 30 (soft cap, Void scales infinitely)
RECRUIT_POOL_SIZE        = 3-5
RECRUIT_REFRESH_HOURS    = 24
AUTO_RESOLVE_TIMEOUT_SECS = 7200 (2 hours)
```

### 1.3 Key Design Decisions

- **Wall-clock time stored as `i64` Unix timestamps** (like `last_save_time` in `GameState`). Avoids `SystemTime` serialization issues.
- **`DeepState` is account-level** (like Haven/Enhancement), persisted to `~/.quest/deep.json`.
- **Mercenary IDs are sequential `u64`** (monotonic counter in `DeepState.next_merc_id`), not UUIDs. Simpler and sufficient.
- **Infrastructure uses a `[Option<InfrastructureType>; 2]` fixed array** per layer — exactly 2 slots, enforced at the type level.
- **Missions store `started_at: i64` and `duration_seconds: u64`** rather than `ends_at`. This allows duration modifiers (Outpost -25%, Saboteur -15%) to be recalculated without stored dependency.
- **`LayerState` uses a `Vec<LayerState>` indexed by layer number (0-based internally, displayed 1-based)**. Only layers that have been reached need entries.

### 1.4 Serde Considerations

- All types derive `Serialize, Deserialize` with `#[serde(default)]` on optional/new fields for backward compatibility.
- `MercenaryStatus::Injured { recover_at: i64 }` stores the wall-clock recovery timestamp.
- Event choices reference archetypes by enum, not string.

---

## Phase 2: Core Logic Modules

**Depends on:** Phase 1 (types)
**Can parallelize internally:** mercenary, mission, layer/economy, and event subsystems are independent.

**Tasks:** #9 (mercenaries), #10 (missions), #11 (layers/economy), #3 (events design)

### 2.1 New Files

| File | Contents |
|------|----------|
| `src/deep/mercenaries.rs` | Merc generation, recruitment, stat scaling, level-up, injury/recovery |
| `src/deep/missions.rs` | Mission generation, ticking, resolution, squad validation |
| `src/deep/events.rs` | Check-in event templates, event resolution, archetype bonuses |
| `src/deep/layers.rs` | Layer progression, infrastructure effects, familiarity, tier queries |
| `src/deep/economy.rs` | Mark earning/spending, guild rank upgrades, cost tables |
| `src/deep/rewards.rs` | Reward generation (XP, items, Stormglass, PR fragments) |

### 2.2 Mercenary System (`mercenaries.rs`)

**Key functions:**
- `generate_mercenary<R: Rng>(rng, archetype, guild_rank, layer_context) -> Mercenary` — stat generation scaled by guild rank
- `generate_recruit_pool<R: Rng>(rng, guild_rank, frontier) -> Vec<Mercenary>` — 3-5 candidates, rarer archetypes at higher ranks
- `recruit(deep: &mut DeepState, index: usize) -> Result<(), RecruitError>` — deducts marks, moves from pool to roster
- `level_up_mercenary(merc: &mut Mercenary)` — XP threshold check, stat growth
- `apply_injury(merc: &mut Mercenary, severity, current_time: i64)` — sets recovery time
- `check_recovery(merc: &mut Mercenary, current_time: i64) -> bool` — transitions Injured -> Available
- `starter_roster<R: Rng>(rng) -> Vec<Mercenary>` — 3 free starter mercs (Vanguard, Scout, Medic)

**Pattern:** All functions take `&mut R where R: Rng` for testability (follows `game_tick()` pattern).

### 2.3 Mission System (`missions.rs`)

**Key functions:**
- `generate_available_missions(deep: &DeepState) -> Vec<MissionTemplate>` — based on frontier, familiarity, guild rank
- `validate_squad(mission: &MissionTemplate, squad: &[&Mercenary]) -> SquadValidation` — checks requirements, computes power rating
- `start_mission(deep: &mut DeepState, template: MissionTemplate, squad: Vec<u64>, current_time: i64) -> Result<Mission, StartError>` — creates mission, marks mercs as OnMission
- `tick_missions(deep: &mut DeepState, current_time: i64) -> Vec<MissionTickResult>` — check all active missions for completion or pending events
- `resolve_mission<R: Rng>(rng, mission: &mut Mission, deep: &mut DeepState) -> MissionOutcome` — final resolution based on squad power, events, familiarity
- `cancel_missions_for_prestige(deep: &mut DeepState, current_time: i64) -> Vec<PartialReward>` — auto-cancel active missions with partial rewards

**Wall-clock ticking model:**
- `tick_missions()` is called from two places:
  1. **On game load** (like offline XP processing) — catches up all missions that completed while offline
  2. **Periodically during play** — every ~10 seconds (100 ticks), check mission progress

### 2.4 Check-In Events (`events.rs`)

**Key functions:**
- `check_event_triggers(mission: &Mission, current_time: i64) -> Option<PendingEvent>` — fires events at 25%/50%/75% progress
- `resolve_event(event: &CheckInEvent, choice_index: usize, squad: &[&Mercenary]) -> EventOutcome` — applies choice effects
- `auto_resolve_pending_events(mission: &mut Mission, current_time: i64) -> Vec<EventOutcome>` — auto-resolve after 2-hour timeout
- `generate_events_for_mission<R: Rng>(rng, mission_type, layer, tier) -> Vec<CheckInEvent>` — 0-5 events per mission

**Event template structure:**
- ~30-50 event templates organized by layer tier (Shallows, Warrens, etc.)
- Each event has 2-4 choices, some gated by archetype
- Events can chain (choice A in event 1 may unlock bonus choice in event 3)

### 2.5 Layer System (`layers.rs`)

**Key functions:**
- `get_layer_tier(layer_num: u32) -> LayerTier` — maps layer number to tier
- `layer_difficulty(layer_num: u32) -> f64` — scaling factor for mission duration/risk/reward
- `familiarity_duration_modifier(familiarity: f64) -> f64` — 0-30% reduction based on 0-100% familiarity
- `infrastructure_effects(layer: &LayerState) -> InfrastructureEffects` — aggregates Outpost/Cache/Watchtower/Bridge bonuses
- `build_infrastructure(deep: &mut DeepState, layer: u32, slot: usize, infra_type: InfrastructureType) -> Result<(), BuildError>` — validates slot availability, deducts marks
- `complete_breakthrough(deep: &mut DeepState, layer: u32)` — marks layer cleared, advances frontier

### 2.6 Economy (`economy.rs`)

**Key functions:**
- `mission_reward_marks(mission_type: MissionType, layer: u32, outcome: MissionOutcome) -> u64` — mark earning table
- `recruit_cost(archetype: MercenaryArchetype, guild_rank: GuildRank) -> u64` — 30-120 marks
- `infrastructure_cost(infra_type: InfrastructureType, layer: u32) -> u64` — 60-150 marks
- `guild_rank_cost(target_rank: GuildRank) -> u64` — 200/500/1200/3000 marks
- `can_upgrade_guild(deep: &DeepState) -> bool` — checks marks + layer requirement
- `upgrade_guild(deep: &mut DeepState) -> Result<(), UpgradeError>` — deducts marks, bumps rank
- `is_free_daily_available(deep: &DeepState, current_time: i64) -> bool` — one free supply run per calendar day

---

## Phase 3: Persistence and Discovery

**Depends on:** Phase 1 (types), Phase 2 (logic for prestige reset handling)
**Tasks:** #12

### 3.1 New Files

| File | Contents |
|------|----------|
| `src/deep/persistence.rs` | `load_deep()`, `save_deep()`, `deep_save_path()` — follows Enhancement pattern exactly |
| `src/deep/discovery.rs` | `deep_discovery_chance()`, `try_discover_deep()` — follows Soulforge discovery pattern |

### 3.2 Persistence Pattern (mirrors `enhancement/persistence.rs`)

```rust
fn deep_save_path() -> io::Result<PathBuf>  // ~/.quest/deep.json
fn load_deep() -> DeepState                  // Returns DeepState::new() on missing/corrupt
fn save_deep(deep: &DeepState) -> io::Result<()>  // Pretty-printed JSON
```

### 3.3 Discovery Pattern (mirrors Soulforge in `tick.rs`)

```rust
fn deep_discovery_chance(prestige_rank: u32) -> f64  // 0.000014 + (rank - 15) * 0.000007
fn try_discover_deep<R: Rng>(deep: &mut DeepState, prestige_rank: u32, rng: &mut R) -> bool
```

### 3.4 Prestige Reset Integration

Add to `perform_prestige()` flow (via a new function or callback pattern):
- `reset_deep_for_prestige(deep: &mut DeepState, current_time: i64)` — clears mercs, marks, active missions (with partial rewards), preserves guild rank, cleared layers, infrastructure, familiarity

### 3.5 Modified Files

| File | Change |
|------|--------|
| `src/main.rs` | Load `deep_state` at startup (like haven/enhancement), save on `deep_changed` flag |
| `src/main_helpers/persistence.rs` | Add `deep` parameter to `save_all()` |
| `src/core/tick_types.rs` | Add `DeepDiscovered` variant to `TickEvent`, add `deep_changed: bool` to `TickResult` |
| `src/core/tick.rs` | Add Stage 13: Deep discovery check (P15+, same guard pattern as Stage 11), add `deep: &mut DeepState` parameter to `game_tick()` |
| `src/character/prestige_actions.rs` | Call `reset_deep_for_prestige()` in prestige flow |
| `src/lib.rs` | Add `pub mod deep;` and re-exports |

---

## Phase 4: UI Overlay and Input Handling

**Depends on:** Phase 1 (types for rendering)
**Can run in parallel with:** Phase 2 (logic) — UI can be built against type stubs

**Tasks:** #5 (design), #13 (UI implementation), #14 (input handling)

### 4.1 New Files

| File | Contents |
|------|----------|
| `src/ui/deep_scene.rs` | Main Deep overlay renderer (delegates to sub-modules) |
| `src/ui/deep_roster.rs` | Roster sub-view — merc list with stats, status, archetype |
| `src/ui/deep_missions.rs` | Mission sub-view — active missions, progress bars, squad details |
| `src/ui/deep_layers.rs` | Layer sub-view — layer list, infrastructure, familiarity |
| `src/ui/deep_events.rs` | Event response sub-view — check-in event choices |
| `src/ui/deep_recruit.rs` | Recruitment sub-view — available candidates, cost display |
| `src/ui/deep_mission_setup.rs` | Mission setup sub-view — mission selection, squad picker |
| `src/input/deep_input.rs` | Deep overlay input routing (follows Haven/Soulforge pattern) |

### 4.2 UI State Types (in `src/deep/types.rs` or `src/input/types.rs`)

```rust
enum DeepView { Main, Roster, Missions, Layers, EventResponse, Recruitment, MissionSetup }
struct DeepUiState {
    open: bool,
    view: DeepView,
    selected_index: usize,        // For list navigation
    squad_selection: Vec<u64>,     // Merc IDs selected for squad
    selected_mission: Option<usize>,
    // ... additional per-view state
}
```

### 4.3 Overlay Integration Pattern

Follows the exact pattern used by Haven/Soulforge/Stormglass:
1. `GameOverlay` enum in `src/input/types.rs` gets a new `Deep` variant
2. `DeepUiState` manages open/closed and sub-view navigation
3. Keybind (e.g., `d` or `g`) toggles the overlay
4. Stats panel shows notification indicator when events are pending (e.g., `[D] Event!`)

### 4.4 Modified Files

| File | Change |
|------|--------|
| `src/input/mod.rs` | Add `deep_input` module, route Deep overlay input |
| `src/input/types.rs` | Add `Deep` variant to `GameOverlay`, add `DeepUiState` |
| `src/ui/mod.rs` | Add `deep_scene` and sub-module declarations |
| `src/ui/stats_panel.rs` | Add Deep notification indicator (pending events, completed missions) |
| `src/main.rs` | Add `deep_ui_state` to game loop, render Deep overlay |
| `src/main_helpers/overlay.rs` | Add Deep overlay to overlay draw dispatch |
| `src/main_helpers/scene.rs` | Include Deep in scene kind checks |
| `src/main_helpers/input_routing.rs` | Route Deep keybind and overlay input |

---

## Phase 5: Game State Integration

**Depends on:** Phase 1 (types), Phase 2 (logic), Phase 3 (persistence)
**Tasks:** #2 (architecture), #10 (mission ticking integration)

### 5.1 Tick Integration

The Deep requires a different ticking model than existing systems:

**Wall-clock ticking** (NOT game-tick-based):
- Missions progress based on `SystemTime::now()` vs `mission.started_at`, not `tick_counter`
- This is similar to `offline.rs` but runs while the game is open too

**Integration point in `game_tick()`:**
- New Stage 13 (or extend Stage 11): Deep discovery check
- New Stage 14: Deep mission ticking — call `tick_missions()` every ~10 seconds (guard with a simple modulo on `tick_counter`)
- Emit new `TickEvent` variants for mission completions, events pending, etc.

**On game load (main.rs):**
- After loading `DeepState`, call `catch_up_missions(deep, current_time)` to process all missions that completed while offline
- Auto-resolve any events that timed out (> 2 hours old)
- Queue completed mission results for display

### 5.2 New TickEvent Variants

```rust
// Add to TickEvent enum in tick_types.rs
DeepDiscovered,
DeepMissionCompleted { mission_type: MissionType, layer: u32, outcome: MissionOutcome, message: String },
DeepEventPending { mission_id: u64, message: String },
DeepEventAutoResolved { message: String },
DeepBreakthroughCompleted { layer: u32, message: String },
```

### 5.3 Modified Files (Integration Touchpoints)

| File | Change |
|------|--------|
| `src/core/tick.rs` | Add `deep: &mut DeepState` parameter, add discovery + mission tick stages |
| `src/core/tick_types.rs` | Add `DeepDiscovered` and mission-related TickEvent variants, add `deep_changed: bool` to TickResult |
| `src/tick_events.rs` | Map new TickEvent variants to combat log entries |
| `src/character/prestige_actions.rs` | Accept `&mut DeepState` in prestige functions for reset |
| `src/main.rs` | Load/save DeepState, pass to `game_tick()`, handle Deep-related events, catch up missions on load |
| `src/main_helpers/persistence.rs` | Include Deep in `save_all()` |
| `src/main_helpers/offline.rs` | Add Deep mission catch-up alongside offline XP |
| `src/bin/simulator.rs` | Add `--deep` flag for simulator (optional, lower priority) |
| `src/achievements/types.rs` | Add Deep-related achievement IDs |
| `src/achievements/handlers.rs` | Add Deep event handlers (on_deep_discovered, on_breakthrough, etc.) |
| `src/achievements/data.rs` | Add Deep achievement definitions |
| `src/utils/debug_menu.rs` | Add Deep debug options (discover, grant marks, force breakthrough) |
| `src/ui/debug_menu_scene.rs` | Add Deep tab to debug menu |

### 5.4 Signature Change: `game_tick()`

The most impactful change is adding `deep: &mut DeepState` to `game_tick()`. This follows the exact pattern used when Enhancement was added (Enhancement was added as a parameter after Haven).

```rust
// Before:
pub fn game_tick<R: Rng>(state, tick_counter, haven, enhancement, achievements, debug_mode, rng) -> TickResult

// After:
pub fn game_tick<R: Rng>(state, tick_counter, haven, enhancement, deep, achievements, debug_mode, rng) -> TickResult
```

**Impact:** Every call site for `game_tick()` must be updated:
- `src/main.rs`
- `src/bin/simulator.rs`
- All integration tests calling `game_tick()` (30+ test files)
- Tests inside `src/core/tick.rs`

---

## Phase 6: Testing

**Progressive — starts after each module is implemented.**
**Tasks:** #15, #16, #17, #18, #19, #20, #21

### 6.1 Unit Tests Per Module

| Module | Test File | Key Test Cases |
|--------|-----------|----------------|
| `deep/mercenaries.rs` | Inline `#[cfg(test)]` | Generate merc stats by archetype, recruit pool generation, level-up thresholds, injury/recovery timing, starter roster composition |
| `deep/missions.rs` | Inline `#[cfg(test)]` | Mission generation by layer/rank, squad validation (requirements/recommendations), mission duration calculations with infrastructure modifiers, resolution outcomes |
| `deep/events.rs` | Inline `#[cfg(test)]` | Event trigger timing (25/50/75%), archetype bonus unlocking, auto-resolve picks safest, event chaining |
| `deep/layers.rs` | Inline `#[cfg(test)]` | Layer tier mapping, familiarity duration modifier curve, infrastructure slot constraints (max 2), breakthrough advances frontier |
| `deep/economy.rs` | Inline `#[cfg(test)]` | Mark earning rates by mission/layer, recruitment costs, guild rank prerequisites, free daily supply run logic |
| `deep/persistence.rs` | Inline `#[cfg(test)]` | Round-trip serialize/deserialize, backward compat with missing fields, corrupt file fallback |
| `deep/discovery.rs` | Inline `#[cfg(test)]` | Discovery probability curve, blocked when not P15+, blocked during active content |

### 6.2 Integration Tests

| Test File | Purpose | Depends On |
|-----------|---------|-----------|
| `tests/deep_mercenary_test.rs` | Merc generation, recruitment, level-up across missions | Phase 2 |
| `tests/deep_mission_test.rs` | Full mission lifecycle: create -> tick -> events -> resolve | Phase 2, 5 |
| `tests/deep_economy_test.rs` | Economy balance: earning vs spending across guild ranks | Phase 2 |
| `tests/deep_persistence_test.rs` | Save/load round-trip, migration from older save formats | Phase 3 |
| `tests/deep_prestige_test.rs` | Prestige reset preserves guild/layers/infra, clears mercs/marks/missions | Phase 3, 5 |
| `tests/deep_discovery_test.rs` | Discovery gating, tutorial flow, initial state | Phase 3, 5 |
| `tests/deep_integration_test.rs` | End-to-end: discover -> recruit -> mission -> breakthrough -> prestige -> resume | Phase 5 |

### 6.3 Testing Strategy

- **All RNG-dependent logic uses generic `<R: Rng>`** for deterministic seeded testing (follows existing `game_tick()` pattern)
- **Wall-clock time is parameterized** — functions accept `current_time: i64` rather than calling `SystemTime::now()`. Tests pass synthetic timestamps.
- **No hardware-dependent timing tests** — test functional outcomes (mission completed? event triggered?) not elapsed wall-clock durations
- **Backward compatibility tests** — deserializing older `DeepState` JSON (with missing new fields) produces valid defaults

---

## Minimum Viable Feature (Smallest Playable Slice)

The MVP is the smallest subset that delivers a playable loop:

### MVP Scope

1. **Discovery** — P15+ tick-based discovery, combat log message
2. **Starter roster** — 3 free mercs (Vanguard, Scout, Medic)
3. **Layer 1-3** (The Shallows) — basic difficulty, no environmental hazards
4. **Supply Run missions only** — 2-4h, safe, no events, no risk
5. **Basic Breakthrough mission** — unlock Layer 2, then Layer 3. 1 event each, simple choices.
6. **Main overlay** — shows guild rank, marks, active missions with progress bars
7. **Roster sub-view** — list mercs, show stats and status
8. **Warband Marks** — earn from supply runs, spend on recruitment
9. **Persistence** — save/load DeepState to `~/.quest/deep.json`
10. **Prestige reset** — clears mercs/marks, preserves cleared layers

### MVP Excludes (Add Later)

- Recon, Expedition, Construction mission types
- Check-in events (beyond basic breakthrough event)
- Infrastructure building
- Familiarity system
- Guild rank upgrades (start at Rank 1 permanently for MVP)
- Recruitment pool (rotating daily candidates) — MVP just gives starter mercs
- Merc injury/loss system
- Rewards flowing to main game (XP, items, Stormglass, PR fragments)
- Achievements
- Debug menu integration
- Simulator integration
- Layer sub-view
- Event response sub-view
- Layers 4+

### MVP Implementation Order

1. `types.rs` — full type definitions (even for post-MVP features, to avoid schema changes)
2. `persistence.rs` + `discovery.rs` — can discover and persist
3. `mercenaries.rs` (starter roster only)
4. `missions.rs` (Supply Run + basic Breakthrough only)
5. `economy.rs` (mark earning from supply runs, recruit cost)
6. `layers.rs` (Layer 1-3 definitions, breakthrough logic)
7. `deep_input.rs` + `deep_scene.rs` (main overlay + roster view)
8. Integration: `tick.rs` discovery, `main.rs` load/save, prestige reset

---

## Risk Assessment

### 1. Wall-Clock Time Implementation (HIGH RISK)

**Risk:** Quest has never used wall-clock time for game mechanics. All existing systems run on the 100ms tick loop. Introducing real-time missions creates new complexity around:
- Time zone handling (use UTC consistently)
- Clock manipulation/cheating (accept it — idle game convention)
- Offline catch-up (missions completed while game was closed)
- Multiple events firing simultaneously on catch-up

**Mitigation:**
- All time as `i64` Unix timestamps (UTC), never `SystemTime` in persisted state
- `tick_missions()` accepts `current_time` parameter (testable, no wall-clock dependency in logic)
- Offline catch-up processes missions in chronological order, not all-at-once
- Auto-resolve always picks safest option — no negative surprise from offline events

### 2. `game_tick()` Signature Change (MEDIUM RISK)

**Risk:** Adding `deep: &mut DeepState` to `game_tick()` touches every call site — main.rs, simulator, and 30+ integration test files.

**Mitigation:**
- This is the same mechanical change that was done when `enhancement` was added
- Can be done as a single atomic commit before any Deep logic is added
- Tests only need a `DeepState::default()` stub initially

### 3. Mission State Persistence Complexity (MEDIUM RISK)

**Risk:** Active missions carry complex state (pending events, squad assignments, timestamps). Serialization bugs could corrupt saves or lose in-progress missions.

**Mitigation:**
- Use `#[serde(default)]` on all fields for backward compatibility
- Round-trip serialization tests for every state combination
- `load_deep()` returns `DeepState::new()` on any parse failure (same as Enhancement)
- Completed missions are moved to a separate `completed_missions` vec (bounded, e.g., last 20)

### 4. UI Complexity — Multiple Sub-Views (MEDIUM RISK)

**Risk:** The Deep overlay has 7 sub-views (Main, Roster, Missions, Layers, EventResponse, Recruitment, MissionSetup). This is significantly more complex than Haven (2 panels) or Soulforge (slot list + animation).

**Mitigation:**
- Each sub-view is a separate file (follows Haven's haven_tree.rs/haven_details.rs pattern)
- MVP ships with only 2 sub-views (Main + Roster)
- State machine in `DeepView` enum prevents invalid transitions
- Sub-views are stateless renderers — all state lives in `DeepUiState`

### 5. Balancing Mission Durations and Rewards (LOW RISK)

**Risk:** Getting the feel right for 2-24 hour missions requires playtesting that can't be unit-tested.

**Mitigation:**
- All duration/reward values are constants in `economy.rs` and `types.rs` — easy to tune
- Simulator can be extended with `--deep` flag for accelerated testing
- MVP starts with conservative values (shorter durations, higher rewards) and tunes down

### 6. Prestige Reset Interaction (LOW RISK)

**Risk:** The split persistence model (some state persists, some resets) could have edge cases.

**Mitigation:**
- The exact same pattern is used by Haven (persists) + GameState (resets)
- `reset_deep_for_prestige()` is a single function with clear documentation
- Integration test covers: prestige -> verify guild/layers preserved, mercs/marks cleared

---

## Task Dependency Graph

```
Phase 1 (Types):
  #1 Design types ──────────────► #8 Implement types.rs
                                      │
                                      ▼
Phase 2 (Logic):            ┌────────────────────────┐
  #3 Event design ─────────►│                        │
  #4 Balance design ────────►│  #9  Mercenaries      │
                             │  #10 Missions          │ (parallelizable)
                             │  #11 Layers/Economy    │
                             │                        │
                             └────────┬───────────────┘
                                      │
Phase 3 (Persistence):               ▼
  #2 Integration arch ─────► #12 Persistence + Discovery
                                      │
Phase 4 (UI):                         │
  #5 UI design ────────────► #13 UI Overlay ──────────┤
                             #14 Input Handling ──────┤
                                                      │
Phase 5 (Integration):                                ▼
                             Full game_tick() integration
                             (combines Phases 2-4)
                                      │
Phase 6 (Testing):                    ▼
  #15 Merc unit tests ◄───── #9
  #16 Mission unit tests ◄── #10
  #17 Economy unit tests ◄── #11
  #18 Persistence tests ◄─── #12
  #19 Discovery flow tests ◄─ #12, #13, #14
  #20 E2E mission tests ◄─── #10, #13
  #21 Prestige reset tests ◄─ #12, #11
```

---

## Complete File Inventory

### New Files (17 files)

| File | Phase | Description |
|------|-------|-------------|
| `src/deep/mod.rs` | 1 | Module declaration and re-exports |
| `src/deep/types.rs` | 1 | All core data structures, constants |
| `src/deep/CLAUDE.md` | 1 | Module documentation |
| `src/deep/mercenaries.rs` | 2 | Merc generation, recruitment, leveling, injury |
| `src/deep/missions.rs` | 2 | Mission lifecycle: generation, ticking, resolution |
| `src/deep/events.rs` | 2 | Check-in event system and templates |
| `src/deep/layers.rs` | 2 | Layer progression, infrastructure, familiarity |
| `src/deep/economy.rs` | 2 | Warband Marks economy, guild ranks, costs |
| `src/deep/rewards.rs` | 2 | Reward generation flowing into existing systems |
| `src/deep/persistence.rs` | 3 | Save/load `~/.quest/deep.json` |
| `src/deep/discovery.rs` | 3 | Discovery roll (P15+ tick-based) |
| `src/ui/deep_scene.rs` | 4 | Main overlay renderer |
| `src/ui/deep_roster.rs` | 4 | Roster sub-view |
| `src/ui/deep_missions.rs` | 4 | Missions sub-view |
| `src/ui/deep_layers.rs` | 4 | Layers sub-view |
| `src/ui/deep_events.rs` | 4 | Event response sub-view |
| `src/ui/deep_recruit.rs` | 4 | Recruitment sub-view |
| `src/ui/deep_mission_setup.rs` | 4 | Mission setup + squad picker |
| `src/input/deep_input.rs` | 4 | Deep overlay input routing |

### New Test Files (7 files)

| File | Phase |
|------|-------|
| `tests/deep_mercenary_test.rs` | 6 |
| `tests/deep_mission_test.rs` | 6 |
| `tests/deep_economy_test.rs` | 6 |
| `tests/deep_persistence_test.rs` | 6 |
| `tests/deep_prestige_test.rs` | 6 |
| `tests/deep_discovery_test.rs` | 6 |
| `tests/deep_integration_test.rs` | 6 |

### Modified Files (19 files)

| File | Phase | Change Summary |
|------|-------|---------------|
| `src/main.rs` | 3, 4, 5 | Add `mod deep`, load/save DeepState, pass to game_tick, render overlay, catch-up missions on load |
| `src/lib.rs` | 1 | Add `pub mod deep;` and re-exports |
| `src/core/tick.rs` | 3, 5 | Add `deep` param to `game_tick()`, add discovery stage, add mission tick stage |
| `src/core/tick_types.rs` | 3, 5 | Add `DeepDiscovered` + mission TickEvent variants, add `deep_changed` flag |
| `src/tick_events.rs` | 5 | Map new Deep TickEvent variants to combat log entries |
| `src/character/prestige_actions.rs` | 3 | Add Deep reset to prestige flow |
| `src/main_helpers/persistence.rs` | 3 | Add `deep` to `save_all()` |
| `src/main_helpers/offline.rs` | 5 | Add Deep mission catch-up on game load |
| `src/main_helpers/overlay.rs` | 4 | Add Deep overlay rendering dispatch |
| `src/main_helpers/scene.rs` | 4 | Include Deep in scene kind checks |
| `src/main_helpers/input_routing.rs` | 4 | Route Deep keybind and overlay input |
| `src/input/mod.rs` | 4 | Add `deep_input` module, route overlay input |
| `src/input/types.rs` | 4 | Add `Deep` variant to `GameOverlay`, add `DeepUiState` |
| `src/ui/mod.rs` | 4 | Add deep scene module declarations |
| `src/ui/stats_panel.rs` | 4 | Add Deep notification indicator |
| `src/achievements/types.rs` | 5 | Add Deep achievement IDs |
| `src/achievements/handlers.rs` | 5 | Add Deep event handlers |
| `src/achievements/data.rs` | 5 | Add Deep achievement definitions |
| `src/utils/debug_menu.rs` | 5 | Add Deep debug options |
| `src/ui/debug_menu_scene.rs` | 5 | Add Deep tab to debug menu |
| `src/bin/simulator.rs` | 5 | Update `game_tick()` call signature (at minimum), optional `--deep` flag |

### Total: ~18 new source files + 7 test files + ~21 modified files

---

## Implementation Sequencing Recommendation

### Suggested Order (Critical Path)

1. **Types first** (#8) — unblocks everything
2. **`game_tick()` signature change** — add `deep: &mut DeepState` parameter with `DeepState::default()` stubs in all call sites. Do this early as a standalone commit to avoid merge conflicts with other work.
3. **Persistence + Discovery** (#12) — small, self-contained, enables testing the discovery flow
4. **Mercenaries + Economy** (#9, #11 partial) — enables recruitment loop
5. **Missions** (#10) — the core gameplay loop
6. **Basic UI** (#13 partial) — main overlay + roster, enough to play
7. **Input handling** (#14) — make it interactive
8. **Events** (#10 partial) — check-in events for missions
9. **Full UI sub-views** (#13 complete) — layers, events, recruitment, mission setup
10. **Achievements + Debug** — polish
11. **Integration tests** — end-to-end validation

### Parallelization Opportunities

- **Tasks #9, #10, #11** can be implemented in parallel by different developers (they share types but have independent logic)
- **Task #13 (UI) and #14 (input)** can proceed in parallel with Phase 2 logic
- **All unit test tasks (#15, #16, #17)** can be written alongside their corresponding modules
- **Integration tests (#18-#21)** must wait for their dependencies but can run in parallel with each other
