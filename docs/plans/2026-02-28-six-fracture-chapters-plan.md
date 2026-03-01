# Six Fracture Chapters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend fracture zones from 3 chapters (Z12-20) to 6 chapters (Z12-30), with one chapter unlocking at each Deep layer tier boundary, aligned 1:1 with Ascension gates.

**Architecture:** Add 3 new `FractureRegion` variants (HollowThrone, WailingReach, OriginWound) and 10 new zone definitions (Z21-30). Shift Black Mouth's unlock layer from 13 to 12. Extend `ZONE_ENEMY_STATS` table with 10 new entries using the existing 1.6x scaling formula. Add 10 new `AchievementId` variants for Z21-30 completion.

**Tech Stack:** Rust, serde (for achievement serialization), ratatui (UI)

**Design doc:** `docs/plans/2026-02-28-six-fracture-chapters-design.md`

---

### Task 1: Extend FractureRegion enum and methods

**Files:**
- Modify: `src/zones/fracture.rs`

**Step 1: Add new enum variants**

Add three new variants to `FractureRegion`:

```rust
pub enum FractureRegion {
    RedFault,
    MirrorScar,
    BlackMouth,
    HollowThrone,
    WailingReach,
    OriginWound,
}
```

**Step 2: Update all match arms**

Every method on `FractureRegion` needs arms for the 3 new variants. Also change `BlackMouth`'s `unlock_layer()` from 13 to 12.

`start_zone_id()`:
```rust
Self::HollowThrone => 21,
Self::WailingReach => 24,
Self::OriginWound => 27,
```

`end_zone_id()`:
```rust
Self::HollowThrone => 23,
Self::WailingReach => 26,
Self::OriginWound => 30,
```

`unlock_layer()` — also change BlackMouth from 13 to 12:
```rust
Self::BlackMouth => 12,
Self::HollowThrone => 18,
Self::WailingReach => 25,
Self::OriginWound => 30,
```

`from_layer()` — also change 13 to 12 for BlackMouth:
```rust
12 => Some(Self::BlackMouth),
18 => Some(Self::HollowThrone),
25 => Some(Self::WailingReach),
30 => Some(Self::OriginWound),
```

`unlock_headline()`:
```rust
Self::HollowThrone => "THE HOLLOW THRONE REVEALS",
Self::WailingReach => "THE WAILING REACH CALLS",
Self::OriginWound => "THE ORIGIN WOUND OPENS",
```

`unlock_atmospheric()`:
```rust
Self::HollowThrone => "Beyond the wound, a kingdom older than the world still waits.",
Self::WailingReach => "Reality forgets itself here. Sound has learned to weep.",
Self::OriginWound => "The first fracture. The wound that was here before the world it broke.",
```

`unlock_mechanical()`:
```rust
Self::HollowThrone => "Zones 21-23 are now reachable beyond the current frontier.",
Self::WailingReach => "Zones 24-26 are now reachable beyond the current frontier.",
Self::OriginWound => "Zones 27-30 are now reachable beyond the current frontier.",
```

`ascension_narrative()`:
```rust
Self::HollowThrone => "The throne's guardians have fallen. Power beyond reckoning yields to your will.",
Self::WailingReach => "The reach has acknowledged you. Even silence bows before your strength.",
Self::OriginWound => "You stand at the source of all breaking. Nothing remains that can challenge you.",
```

`ascension_level_unlocked()`:
```rust
Self::HollowThrone => 4,
Self::WailingReach => 5,
Self::OriginWound => 6,
```

`unlock_log_line()`:
```rust
Self::HollowThrone => "The Hollow Throne has revealed itself beneath the wound.",
Self::WailingReach => "The Wailing Reach calls from the boundary of existence.",
Self::OriginWound => "The Origin Wound has opened at the source of all fractures.",
```

`unlock_ticker_text()`:
```rust
Self::HollowThrone => "Hollow Throne available",
Self::WailingReach => "Wailing Reach available",
Self::OriginWound => "Origin Wound available",
```

**Step 3: Update existing unit tests**

Update `test_unlock_layers` — BlackMouth now returns 12, not 13:
```rust
assert_eq!(FractureRegion::BlackMouth.unlock_layer(), 12);
```

Update `test_region_from_layer` — layer 12 now maps to BlackMouth, layer 13 returns None:
```rust
assert_eq!(FractureRegion::from_layer(12), Some(FractureRegion::BlackMouth));
assert_eq!(FractureRegion::from_layer(13), None);
```

Add tests for new variants covering: `start_zone_id`, `end_zone_id`, `unlock_layer`, `from_layer`, serde round-trip.

**Step 4: Run tests**

Run: `cargo test --lib zones -- --quiet`

**Step 5: Commit**

```
feat(zones): add HollowThrone, WailingReach, OriginWound fracture regions
```

---

### Task 2: Extend ZONE_ENEMY_STATS table and constants

**Files:**
- Modify: `src/core/constants.rs`

**Step 1: Compute new zone stats**

Existing Z20 stats: `(343597, 27488, 34360, 5498, 17180, 2062)`. Each zone scales by 1.6x from the previous. Compute Z21-Z30 by multiplying each value by 1.6 iteratively and rounding to nearest integer:

```rust
// Zone 21-30: continued 1.6x exponential scaling
(549755, 43980, 54976, 8796, 27488, 3299),     // Zone 21: Sunken Processional
(879608, 70369, 87961, 14074, 43981, 5278),     // Zone 22: The Pale Archive
(1407373, 112590, 140737, 22518, 70370, 8445),  // Zone 23: The Hollow Throne
(2251797, 180144, 225180, 36029, 112591, 13512),// Zone 24: The Stillborn Sea
(3602875, 288230, 360288, 57646, 180146, 21619),// Zone 25: Resonance Fault
(5764601, 461168, 576460, 92234, 288233, 34590),// Zone 26: The Wailing Reach
(9223361, 737869, 922336, 147574, 461173, 55344),// Zone 27: The Scar Root
(14757378, 1180590, 1475738, 236118, 737877, 88551),// Zone 28: Echoing Abyss
(23611805, 1888944, 2361181, 377789, 1180603, 141682),// Zone 29: Threshold of Silence
(37778888, 3022311, 3777889, 604462, 1888965, 226691),// Zone 30: The Origin Wound
```

To compute these correctly: write a small script or compute in the test. The formula is `Z[n] = round(Z[n-1] * 1.6)` applied to each of the 6 tuple values independently. Use the existing Z20 values as the base.

**Important:** The array size changes from `[(u32, u32, u32, u32, u32, u32); 20]` to `[(u32, u32, u32, u32, u32, u32); 30]`.

**Step 2: Update LAST_FRACTURE_ZONE_ID**

```rust
pub const LAST_FRACTURE_ZONE_ID: u32 = 30;
```

**Step 3: Update tests**

- `test_zone_enemy_stats_has_20_entries` → rename to `test_zone_enemy_stats_has_30_entries`, assert `.len() == 30`
- `test_fracture_constants_exist` → update `LAST_FRACTURE_ZONE_ID` assertion to 30
- Add `test_fracture_zone_stats_zone_30` to verify Z30 values
- Verify Z21 is exactly `round(Z20 * 1.6)` for each field

**Step 4: Run tests**

Run: `cargo test --lib core::constants -- --quiet`

**Step 5: Commit**

```
feat(constants): extend ZONE_ENEMY_STATS table to 30 zones
```

---

### Task 3: Add 10 new zone definitions in data.rs

**Files:**
- Modify: `src/zones/data.rs`

**Step 1: Add Zone 21-30 definitions**

Add 10 new `Zone` entries after Zone 20 in the `ALL_ZONES` vec. Each zone has `prestige_requirement: 0`, `requires_weapon: false`, `weapon_name: None`, and 5 subzones. Follow the same patterns as existing fracture zones.

Level ranges (continuing the 15-level-per-zone pattern):
- Z21: 300-315, Z22: 315-330, Z23: 330-345
- Z24: 345-360, Z25: 360-375, Z26: 375-390
- Z27: 390-405, Z28: 405-420, Z29: 420-435
- Z30: 435-MAX (`u32::MAX`)

Use zone names, subzone names, descriptions, and boss names from the design doc (`docs/plans/2026-02-28-six-fracture-chapters-design.md`). Three chapter comment markers:
```rust
// ── Chapter 4: The Hollow Throne (Zones 21-23) ─────────────────
// ── Chapter 5: The Wailing Reach (Zones 24-26) ─────────────────
// ── Chapter 6: The Origin Wound (Zones 27-30) ──────────────────
```

Cap zone bosses (Z23, Z26, Z30) have `is_zone_boss: true` on their last subzone. The last subzone of each zone shares the zone's name (matching existing pattern).

**Step 2: Update tests**

- `test_zone_count`: change assertion from 20 to 30
- Add tests for new zone ranges: `test_hollow_throne_zone_range`, `test_wailing_reach_zone_range`, `test_origin_wound_zone_range`
- Verify subzone count (5 per zone) for new zones

**Step 3: Run tests**

Run: `cargo test --lib zones::data -- --quiet`

**Step 4: Commit**

```
feat(zones): add zone definitions for Z21-30 (chapters 4-6)
```

---

### Task 4: Add enemy name prefixes/suffixes for Z21-30

**Files:**
- Modify: `src/combat/enemy_generation.rs`

**Step 1: Add prefix/suffix entries for zones 21-30**

In `get_zone_enemy_prefixes()`:
```rust
21 => &["Pilgrim", "Amber", "Candlebone", "Crown", "Processional"],
22 => &["Index", "Sealed", "Theorem", "Forbidden", "Pale"],
23 => &["Echo", "Dust", "Void", "Coronation", "Hollow"],
24 => &["Tideless", "Brine", "Calcified", "Abyssal", "Stillborn"],
25 => &["Harmonic", "Choir", "Oscillation", "Petrified", "Resonance"],
26 => &["Fraying", "Liminal", "Static", "Flickering", "Wailing"],
27 => &["Root", "Splinter", "Taproot", "Fossilized", "Scar"],
28 => &["Echo", "Reverberation", "Temporal", "Infinite", "Ancient"],
29 => &["Dimming", "Hush", "Soundshadow", "Silent", "Fading"],
30 => &["Fissure", "Primordial", "Unbroken", "Wound", "Origin"],
```

In `get_zone_enemy_suffixes()`:
```rust
21 => &["Sentinel", "Warden", "Knight", "Colossus", "Procession"],
22 => &["Wraith", "Censor", "Construct", "Eater", "Archivist"],
23 => &["Warden", "Chancellor", "Guardian", "Absence", "Sovereign"],
24 => &["Wanderer", "Phantom", "Leviathan", "Depthless", "Mother"],
25 => &["Hound", "Dissonant", "Resonant", "Warden", "Chorus"],
26 => &["Stalker", "Undefined", "Bloom", "Flickerer", "Voice"],
27 => &["Creeper", "Horror", "Warden", "Rupture", "Root"],
28 => &["Echo", "Dweller", "Noise", "Once-Slain", "Reverberation"],
29 => &["Walker", "Muted", "Beast", "End", "Warden"],
30 => &["Guardian", "Titan", "Unbroken", "Heart", "Final"],
```

**Step 2: Update the test that checks zone 12-20 prefixes**

Extend the range from `12..=20` to `12..=30`:
```rust
for zone_id in 12..=30 {
```

**Step 3: Run tests**

Run: `cargo test --lib combat::enemy_generation -- --quiet`

**Step 4: Commit**

```
feat(combat): add enemy name prefixes/suffixes for Z21-30
```

---

### Task 5: Add achievement variants for Z21-30

**Files:**
- Modify: `src/achievements/types.rs`
- Modify: `src/achievements/data.rs`
- Modify: `src/achievements/handlers.rs`

**Step 1: Add AchievementId variants**

In `src/achievements/types.rs`, after `FractureZone20`:
```rust
FractureZone21, // Processional's End
FractureZone22, // Archive Breaker
FractureZone23, // Throne Claimer
FractureZone24, // Sea Walker
FractureZone25, // Silence Breaker
FractureZone26, // Edge Runner
FractureZone27, // Root Cutter
FractureZone28, // Echo Silencer
FractureZone29, // Last Sound
FractureZone30, // Wound Closer
```

**Step 2: Add AchievementDef entries**

In `src/achievements/data.rs`, after the FractureZone20 entry, add 10 new entries. Follow the existing pattern (name, description, category Progression, point value). Use point values consistent with existing fracture achievements (check what Z12-20 use — likely 50 or 100 points).

Example:
```rust
AchievementDef {
    id: AchievementId::FractureZone21,
    name: "Processional's End",
    description: "Defeat the final boss of Zone 21 (Sunken Processional)",
    category: AchievementCategory::Progression,
    points: 100,
    grants_title: false,
},
```

Assign `grants_title: true` to FractureZone30 ("Wound Closer") as a prestige title for completing all content.

Also add FractureZone21-30 to the `PROGRESSION_ACHIEVEMENTS` array.

**Step 3: Add handler mappings**

In `src/achievements/handlers.rs`, in the `on_zone_completed` match (or equivalent), add:
```rust
21 => Some(AchievementId::FractureZone21),
22 => Some(AchievementId::FractureZone22),
23 => Some(AchievementId::FractureZone23),
24 => Some(AchievementId::FractureZone24),
25 => Some(AchievementId::FractureZone25),
26 => Some(AchievementId::FractureZone26),
27 => Some(AchievementId::FractureZone27),
28 => Some(AchievementId::FractureZone28),
29 => Some(AchievementId::FractureZone29),
30 => Some(AchievementId::FractureZone30),
```

**Step 4: Run tests**

Run: `cargo test --lib achievements -- --quiet`

**Step 5: Commit**

```
feat(achievements): add FractureZone21-30 achievements
```

---

### Task 6: Update integration tests

**Files:**
- Modify: `tests/fracture_zones_test.rs`
- Modify: `tests/fracture_deep_test.rs`
- Modify: `tests/ascension_test.rs`

**Step 1: Update fracture_zones_test.rs**

Add zone progression tests for Z21-30. Follow existing test patterns (travel_to zone, defeat bosses, verify advancement). Test cap zone cycling for Z23, Z26, Z30.

**Step 2: Update fracture_deep_test.rs**

Add integration tests for new breakthrough layers:
- Layer 18 breakthrough → Hollow Throne (Z21-23, cap = 23)
- Layer 25 breakthrough → Wailing Reach (Z24-26, cap = 26)
- Layer 30 breakthrough → Origin Wound (Z27-30, cap = 30)
- Layer 12 breakthrough → Black Mouth (was layer 13, now 12)

Update existing tests if they reference layer 13 for Black Mouth.

**Step 3: Update ascension_test.rs**

Verify `can_ascend` still works correctly. The deep gates haven't changed (`[3, 7, 12, 18, 25, 30]`) — the only change is that BlackMouth's `unlock_layer()` moved from 13 to 12, which is in the fracture system, not the ascension system. However, verify no tests depend on the old layer 13 mapping.

**Step 4: Run all tests**

Run: `cargo test -- --quiet`

**Step 5: Commit**

```
test: add integration tests for Z21-30 fracture chapters
```

---

### Task 7: Update debug menu for new zones

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Check if travel-to-zone debug action supports Z21-30**

The debug menu has a "travel to fracture zone" action. If it uses a hardcoded range (e.g., 12-20), extend it to 12-30. If it iterates zones dynamically, no change needed.

**Step 2: Run tests**

Run: `cargo clippy --all-targets -- -D warnings`

**Step 3: Commit (if changes made)**

```
feat(debug): extend fracture zone travel to Z21-30
```

---

### Task 8: Update documentation

**Files:**
- Modify: `CLAUDE.md` (root)
- Modify: `src/zones/CLAUDE.md`
- Modify: `src/ascension/CLAUDE.md`
- Modify: `src/deep/CLAUDE.md`
- Modify: `src/achievements/CLAUDE.md`
- Modify: `src/core/CLAUDE.md`
- Modify: `src/combat/CLAUDE.md`

**Step 1: Root CLAUDE.md**

- Update zone tier table: add Ch.4, Ch.5, Ch.6 entries
- Update zone count (20 → 30 zones)
- Update "Fracture zone unlock" constant line to include layers 18, 25, 30
- Update project structure zone count references
- Update achievement count if referenced

**Step 2: Module CLAUDE.md files**

- `src/zones/CLAUDE.md`: Add new chapters to zone tier table, update FractureRegion docs
- `src/ascension/CLAUDE.md`: Note 1:1 alignment with all 6 chapters
- `src/deep/CLAUDE.md`: Note new chapter unlocks at all tier boundaries
- `src/achievements/CLAUDE.md`: Update fracture zone achievement range (Z12-Z30)
- `src/core/CLAUDE.md`: Update ZONE_ENEMY_STATS docs (array size 30)
- `src/combat/CLAUDE.md`: Note Z21-30 enemy name prefixes/suffixes

**Step 3: Commit**

```
docs: update CLAUDE.md files for six fracture chapters
```

---

### Task 9: Final verification

**Step 1: Run full CI check**

Run: `make check`

This runs: format check, clippy, all tests, build, audit.

**Step 2: Verify no stale references**

Run: `grep -ri "LAST_FRACTURE_ZONE_ID.*20\b" src/` — should return nothing.
Run: `grep -ri "zones 1-20\b" src/` — verify zone count references are updated.

**Step 3: Verify fracture region completeness**

Confirm `grep -c "from_layer" src/zones/fracture.rs` shows all 6 mapping entries.

---

## Summary of Changes

| Area | Files | What changes |
|------|-------|-------------|
| FractureRegion | `src/zones/fracture.rs` | 3 new variants, BlackMouth L13→L12, all method arms |
| Zone stats | `src/core/constants.rs` | Array 20→30 entries, LAST_FRACTURE_ZONE_ID 20→30 |
| Zone defs | `src/zones/data.rs` | 10 new Zone structs with 50 subzones |
| Enemy names | `src/combat/enemy_generation.rs` | 10 new prefix/suffix entries |
| Achievements | `src/achievements/{types,data,handlers}.rs` | 10 new FractureZone variants |
| Tests | `tests/{fracture_zones,fracture_deep,ascension}_test.rs` | New integration tests |
| Debug | `src/utils/debug_menu.rs` | Extend zone travel range |
| Docs | 7 CLAUDE.md files | Zone counts, tier tables, references |
