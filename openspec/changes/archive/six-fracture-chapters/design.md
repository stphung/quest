> Backported design record. Sources: docs/plans/2026-02-28-six-fracture-chapters-design.md.

## 2026-02-28-six-fracture-chapters-design.md

# Design: Six Fracture Chapters (Zones 12-30)

## Overview

Extend the fracture zone system from 3 chapters (Z12-20) to 6 chapters (Z12-30), with one chapter unlocking at the end of each Deep layer tier. Every Ascension gate aligns 1:1 with a chapter unlock, so players always have new zones to use their newly earned combat power in.

## Structural Change

### Before (3 chapters, Z12-20)

| Layer | Chapter | Zones | Ascension |
|-------|---------|-------|-----------|
| 3 | Red Fault | Z12-14 | I (2x) |
| 7 | Mirror Scar | Z15-17 | II (4x) |
| 13 | Black Mouth | Z18-20 | III (8x) |
| 18 | *(nothing)* | — | IV (16x) |
| 25 | *(nothing)* | — | V (32x) |
| 30 | *(nothing)* | — | VI (64x) |

Layers 14-30 had no zone unlocks — 17 Deep layers with no new surface content.

### After (6 chapters, Z12-30)

| Layer | Chapter | Zones | Ascension |
|-------|---------|-------|-----------|
| 3 | Ch.1 The Red Fault | Z12-14 (3 zones) | I (2x) |
| 7 | Ch.2 The Mirror Scar | Z15-17 (3 zones) | II (4x) |
| 12 | Ch.3 The Black Mouth | Z18-20 (3 zones) | III (8x) |
| 18 | Ch.4 The Hollow Throne | Z21-23 (3 zones) | IV (16x) |
| 25 | Ch.5 The Wailing Reach | Z24-26 (3 zones) | V (32x) |
| 30 | Ch.6 The Origin Wound | Z27-30 (4 zones) | VI (64x) |

**Total:** 19 fracture zones, 95 subzones (5 per zone), ending at Zone 30.

### Key Changes

- **Black Mouth moves from Layer 13 → Layer 12** to align with Ascension III deep gate
- Each chapter unlocks at the **end of a Deep tier**: Shallows (3), Warrens (7), Hollows (12), Sunken Reach (18), Abyss (25), Void/Gateway (30)
- Ch.6 gets 4 zones (Z27-30) to end at the round Zone 30

## Narrative Arc

The six chapters tell a story through geography — each pushes deeper into the wound:

1. **Red Fault** — The wound is *burning*. Fire, geological violence, blood.
2. **Mirror Scar** — The wound *refracts reality*. Light bends, reflections lie.
3. **Black Mouth** — The wound *hungers*. Void, consumption, organic darkness.
4. **Hollow Throne** — Beyond the void, something *ancient* waits. A preserved kingdom older than the fractures.
5. **Wailing Reach** — Reality itself *unravels*. Sound crystallizes, existence flickers.
6. **Origin Wound** — The *source*. The first fracture — older than the world it broke.

## Chapter Designs

### Ch.1-3: Existing (unchanged except Black Mouth layer shift)

Zones 12-20 remain as-is. Only change: `FractureRegion::BlackMouth.unlock_layer()` returns 12 instead of 13.

### Ch.4: The Hollow Throne (Z21-23, Deep Layer 18, Ascension IV)

*Beyond the devouring void of the Black Mouth, something waited. A kingdom older than the fractures — perhaps older than the world above. Their halls are preserved in crystallized silence. Their throne sits empty. Their guardians still keep watch.*

**Unlock modal:**
- Headline: `THE HOLLOW THRONE REVEALS`
- Atmospheric: *"Beyond the wound, a kingdom older than the world still waits."*
- Ascension: *"The throne's guardians have fallen. Power beyond reckoning yields to your will."*

#### Zone 21: Sunken Processional (lvl 300-315)

*A grand ceremonial road descending into a buried kingdom. The walls are crystallized amber. Faded grandeur preserved in perfect silence.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Pilgrim's Descent | Processional Sentinel | A spiraling road worn smooth by countless feet that walked it before the world above existed. |
| 2 | Amber Gallery | Entombed Warden | The walls are thick with fossilized amber. Shapes move inside it — not insects, but people. |
| 3 | Candlebone Walk | Tallow Knight | Pillars of luminescent bone light the way. They have burned for longer than living memory. |
| 4 | Crown Gate | Gate Colossus | A gate carved from a single gemstone the size of a cathedral. It is open. It was never meant to be. |
| 5 | Sunken Processional | **The Stone Procession** | The road ends at a vast courtyard. The sky above is stone. The silence is absolute. |

#### Zone 22: The Pale Archive (lvl 315-330)

*Libraries of crystallized knowledge carved into burial niches. Each niche holds a crystal tablet instead of a body. The guardians here protect not from invasion but from understanding.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Index Catacombs | Scroll Wraith | Endless shelves carved into burial niches. Each holds a crystal tablet instead of a body. |
| 2 | Hall of Sealed Words | The Censor | Words too dangerous to speak are preserved here in solid form. They hum. |
| 3 | Theorem Vault | Paradox Construct | Mathematical proofs have crystallized into physical objects. Some of them are wrong. Those ones move. |
| 4 | The Forbidden Index | Memory Eater | The section that was locked when the kingdom still lived. The lock is broken now. |
| 5 | The Pale Archive | **The Archivist Eternal** | The central reading room. The tablets here contain the kingdom's final entry. It is one word, repeated. |

#### Zone 23: The Hollow Throne (lvl 330-345, cap zone)

*The seat of power. The palace is intact but empty. Whatever sat here left willingly — or was removed. The room defends its throne as though something still sits there.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Antechamber of Echoes | Echo Warden | A waiting room where visitors once stood. Their conversations still play on loop. |
| 2 | Council of Dust | The Dustbound Chancellor | A round table where advisors sat. Their chairs hold impressions. Their dust holds opinions. |
| 3 | Royal Causeway | Void Mirror Guardian | A bridge of polished obsidian leading to the inner sanctum. It reflects nothing. |
| 4 | Coronation Steps | The Crowned Absence | Steps leading to the throne. Each is engraved with a name. The final name has been scratched out. |
| 5 | The Hollow Throne | **The Hollow Sovereign** | The throne is empty but the room defends it as though something still sits there. |

### Ch.5: The Wailing Reach (Z24-26, Deep Layer 25, Ascension V)

*Past the fallen kingdom, the rules break down. Sound has weight. Light has temperature. Distance lies. This is where the world ends — not with violence, but with silence. The wailing is not pain. It is the sound reality makes when it stops being sure it exists.*

**Unlock modal:**
- Headline: `THE WAILING REACH CALLS`
- Atmospheric: *"Reality forgets itself here. Sound has learned to weep."*
- Ascension: *"The reach has acknowledged you. Even silence bows before your strength."*

#### Zone 24: The Stillborn Sea (lvl 345-360)

*An underground ocean that never came alive. No waves, no life, no temperature. Walking on its surface is like walking on solid grief.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Tideless Shore | Drowned Wanderer | A beach of grey sand touching water that has never moved. No wind has ever blown here. |
| 2 | Salt Meridian | Brine Phantom | A line of crystallized salt runs across the sea. On either side, the water is different colors. |
| 3 | The Unborn Reef | Calcified Leviathan | Coral formations that grew and then stopped. They are shaped like things that were never alive. |
| 4 | Abyssal Shelf | The Depthless | The sea floor drops away into depths that have no bottom. The water below is not water. |
| 5 | The Stillborn Sea | **Mother of Still Waters** | The center of the sea. The water is perfectly transparent. What lies beneath is visible and should not be. |

#### Zone 25: Resonance Fault (lvl 360-375)

*Sound has crystallized into physical structures. Footsteps create visible ripples. Voices from ages past still echo, frozen in crystal.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Harmonic Crust | Frequency Hound | The ground vibrates at a frequency just below hearing. Your teeth ache. Your bones hum. |
| 2 | Choir Stone | The Dissonant | Stone pillars that sing when the wind passes through. There is no wind. |
| 3 | Oscillation Bridge | Null Resonant | A bridge that exists only when a specific note is sustained. Silence drops you into nothing. |
| 4 | The Petrified Scream | Scream Warden | A scream from millennia ago, frozen in crystal. It is still screaming. |
| 5 | Resonance Fault | **The Unheard Chorus** | The source of all sound in this place. A crack that hums with every frequency at once. |

#### Zone 26: The Wailing Reach (lvl 375-390, cap zone)

*The boundary between existence and void. Things here flicker between real and not. The wailing is the sound of matter deciding whether it exists.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Fraying Edge | Border Stalker | The ground has gaps — not holes, but places where the ground simply isn't. They move. |
| 2 | Liminal Corridor | The Undefined | A hallway always the same length no matter how far you walk. The walls are forgetting to be walls. |
| 3 | Static Garden | Entropy Bloom | A garden where flowers are made of frozen static. They grow, bloom, and die in visual noise. |
| 4 | The Last Light | The Flickering One | A single point of light hovering in vast dark. The last thing here certain it exists. |
| 5 | The Wailing Reach | **The Voice at the Edge** | The boundary itself. One step more and things stop being things. The wailing is loudest here. |

### Ch.6: The Origin Wound (Z27-30, Deep Layer 30, Ascension VI)

*The source of all fractures. Not damage — memory. The world remembering what it was before it learned to be solid. The fractures are not breaks in reality. They are reality remembering that it was never whole. You are not descending into a wound. You are arriving at a truth.*

Four zones — the grand finale earns an extra zone.

**Unlock modal:**
- Headline: `THE ORIGIN WOUND OPENS`
- Atmospheric: *"The first fracture. The wound that was here before the world it broke."*
- Ascension: *"You stand at the source of all breaking. Nothing remains that can challenge you."*

#### Zone 27: The Scar Root (lvl 390-405)

*Petrified tendrils of primordial fracture energy radiate outward like the root system of a dead tree. This is where the cracks began spreading.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Rootthread Pass | Root Creeper | Thin veins of fracture energy wind through stone. They pulse with a rhythm older than heartbeats. |
| 2 | Splinter Nest | Nestbound Horror | A tangle of petrified fracture-roots knotted into a den. Something has been living in the cracks. |
| 3 | Taproot Chamber | Taproot Warden | A central root as wide as a river descends into darkness. It feeds on something below. |
| 4 | Fossilized Eruption | The Calcified Rupture | A frozen moment: the instant the first crack split the world. The energy is still here, arrested. |
| 5 | The Scar Root | **Root of All Scars** | The root's origin point. Not rock, not energy — the idea of breaking, made physical. |

#### Zone 28: Echoing Abyss (lvl 405-420)

*A vast emptiness where everything reverberates forever. Actions echo forward and backward in time. Your footsteps arrive before you do.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | First Echo | Precursor Echo | Your footsteps arrive before you do. The echoes here precede their causes. |
| 2 | Reverberation Well | Well Dweller | A pit where sound falls forever. Drop a stone and you hear it land yesterday. |
| 3 | Temporal Silt | The Ancient Noise | The ground is made of compacted echoes. Dig and you hear conversations from before language. |
| 4 | The Infinite Repeat | The Once-Slain | A chamber where everything happens again. The monsters here have died a thousand times and remember each. |
| 5 | Echoing Abyss | **The Eternal Reverberation** | The center of all echoes. Here, the echo and the original are the same event. |

#### Zone 29: Threshold of Silence (lvl 420-435)

*The last place where sound and light exist. Beyond this, there is only the wound.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | Dimming Walk | Fade Walker | Each step forward, the light dims. Not because darkness grows, but because light has less reason to be. |
| 2 | Hush Fields | The Muted | A plain where sound softens with every yard. At the center, even thought is quiet. |
| 3 | Shadow of Sound | Soundshadow Beast | Not shadow from light — shadow from sound. An area where noise casts visible darkness. |
| 4 | The Final Frequency | Frequency's End | A single note hangs in the air. The last sound that will ever be heard in this place. |
| 5 | Threshold of Silence | **The Silent Warden** | The threshold. One step more and there is no more sound, no more light. Just the wound. |

#### Zone 30: The Origin Wound (lvl 435+, cap zone)

*The primordial break. Not a place but an event fossilized into geography. The wound is open, patient, and eternal. It was here before the world it broke.*

| # | Subzone | Boss | Description |
|---|---------|------|-------------|
| 1 | The First Crack | Fissure Guardian | The original fracture. Barely wider than a sword, but everything else grew from it. |
| 2 | Primordial Scar | Scar Titan | The scar tissue of the universe. Layered over itself a billion times, never healing, always growing. |
| 3 | Memory of Wholeness | The Unbroken | A pocket where reality remembers being unbroken. It hurts to be here. Wholeness is heavy. |
| 4 | Wound's Heart | Heart of the Wound | The center of the original break. Not stone, not void — a state of matter that exists only here. |
| 5 | The Origin Wound | **The First and Final** | The wound itself. Open, patient, eternal. Something guards the entrance to eternity. |

## Achievements (9 new)

Continuing the existing pattern of one achievement per fracture zone:

| Achievement | Zone | Category |
|-------------|------|----------|
| FractureZone21 | Sunken Processional | Progression |
| FractureZone22 | The Pale Archive | Progression |
| FractureZone23 | The Hollow Throne | Progression |
| FractureZone24 | The Stillborn Sea | Progression |
| FractureZone25 | Resonance Fault | Progression |
| FractureZone26 | The Wailing Reach | Progression |
| FractureZone27 | The Scar Root | Progression |
| FractureZone28 | Echoing Abyss | Progression |
| FractureZone29 | Threshold of Silence | Progression |
| FractureZone30 | The Origin Wound | Progression |

Plus new Ascension milestones if not already present (AscensionIV, AscensionV — AscensionVI already implied by existing AscensionVI achievement).

## Stat Scaling

Enemy stats scale at 1.6x per zone from Zone 11 base (`FRACTURE_ZONE_STAT_MULTIPLIER`):

| Zone | Raw Scale vs Z11 | Ascension | Ratio (enemies/asc) |
|------|------------------|-----------|---------------------|
| Z14 cap | ~6.6x | I (2x) | 3.3x |
| Z17 cap | ~17x | II (4x) | 4.3x |
| Z20 cap | ~44x | III (8x) | 5.5x |
| Z23 cap | ~113x | IV (16x) | 7.1x |
| Z26 cap | ~292x | V (32x) | 9.1x |
| Z30 cap | ~1,207x | VI (64x) | 18.9x |

The ratio grows, meaning later chapters require more prestige investment alongside Ascension. This is intentional — Ascension alone should not trivialize content.

## FractureRegion Enum Changes

```rust
pub enum FractureRegion {
    RedFault,       // Z12-14, Layer 3
    MirrorScar,     // Z15-17, Layer 7
    BlackMouth,     // Z18-20, Layer 12 (moved from 13)
    HollowThrone,   // Z21-23, Layer 18
    WailingReach,   // Z24-26, Layer 25
    OriginWound,    // Z27-30, Layer 30
}
```

### Method updates needed

| Method | New entries |
|--------|-----------|
| `start_zone_id()` | HollowThrone→21, WailingReach→24, OriginWound→27 |
| `end_zone_id()` | HollowThrone→23, WailingReach→26, OriginWound→30 |
| `unlock_layer()` | BlackMouth→12, HollowThrone→18, WailingReach→25, OriginWound→30 |
| `from_layer()` | 12→BlackMouth, 18→HollowThrone, 25→WailingReach, 30→OriginWound |
| `unlock_headline()` | New strings per chapter |
| `unlock_atmospheric()` | New strings per chapter |
| `unlock_mechanical()` | New strings per chapter |
| `ascension_narrative()` | New strings per chapter |
| `ascension_level_unlocked()` | HollowThrone→4, WailingReach→5, OriginWound→6 |
| `unlock_log_line()` | New strings per chapter |
| `unlock_ticker_text()` | New strings per chapter |

### Ascension deep gate alignment fix

`ASCENSION_DEEP_GATES` in `src/ascension/types.rs` changes from `[3, 7, 12, 18, 25, 30]` — already aligned! The only change is Black Mouth's `unlock_layer()` moving from 13 to 12.

## Constants Changes

- `LAST_FRACTURE_ZONE_ID`: 20 → 30
- `FIRST_FRACTURE_ZONE_ID`: stays at 12
- Zone enemy stats table: extend with entries for Z21-30
- `default_fracture_zone_cap()`: stays at 11 (Expanse only)

## Files Affected

### Zone data
- `src/zones/data.rs` — Add 10 new Zone definitions (Z21-30)
- `src/zones/fracture.rs` — Add 3 FractureRegion variants, update all methods
- `src/zones/CLAUDE.md` — Update zone tier table

### Achievements
- `src/achievements/types.rs` — Add FractureZone21-30 variants
- `src/achievements/data.rs` — Achievement descriptions for new zones
- `src/achievements/handlers.rs` — Unlock triggers for Z21-30

### Constants
- `src/core/constants.rs` — Update LAST_FRACTURE_ZONE_ID, add enemy stat entries for Z21-30

### Combat
- `src/combat/enemy_generation.rs` — Stat scaling continues to work via existing FRACTURE_ZONE_STAT_MULTIPLIER formula (no changes needed if formula-driven)

### UI
- `src/ui/combat_scene.rs` — Unlock modal already handles any FractureRegion (no changes if data-driven)
- `src/ui/stats_panel.rs` — Zone display already handles any zone ID (no changes expected)

### Documentation
- `CLAUDE.md` (root) — Update zone tier table, zone count, key constants
- `src/zones/CLAUDE.md` — Add new chapters to tier table
- `src/ascension/CLAUDE.md` — Note alignment
- `src/deep/CLAUDE.md` — Note new chapter unlocks at tier boundaries

### Tests
- `src/zones/fracture.rs` tests — Update for new variants and BlackMouth layer change
- `tests/fracture_zones_test.rs` — Add tests for Z21-30 progression
- `tests/fracture_deep_test.rs` — Add integration tests for new breakthrough layers
- `tests/ascension_test.rs` — Update if BlackMouth layer change affects any test
- `src/zones/data.rs` tests — Update zone count assertion (20 → 30)
