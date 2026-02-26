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
