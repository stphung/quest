# Loom of Worlds — Sustained Rate Pattern Redesign

## Overview

Redesign the Woven Pattern system from accumulated totals to sustained production rates. Patterns require the player to maintain a minimum flow rate for a duration, proving their production network works reliably. Expand from 18 to 28 patterns with full resource coverage.

## Core Mechanic: Sustained Flow Rates

### How It Works

Each pattern requirement specifies:
- **Rate threshold** (units/hr) — minimum production rate the player must sustain
- **Sustain duration** (hours) — how long the rate must be maintained

The player builds their production network to meet the threshold, verifies rates are green, then walks away. The Loom drinks continuously in the background.

### Measurement

**60-second rolling window average** (600 ticks at 100ms/tick). Smooths production spikes without being sluggish. Running sum maintained for O(1) per tick.

```rust
struct RateTracker {
    buffer: VecDeque<f64>,   // last 600 ticks of production
    window_size: usize,      // 600
    sum: f64,                // running sum
}
```

### Failure Model: Simple Pause

When the rate drops below the threshold:
- **Progress timer freezes** — does not advance, does not decay
- When rate recovers, timer resumes exactly where it left off
- **Progress is never lost**, only paused

This is idle-friendly: if the player walks away and something breaks, they fix it when they come back and the timer picks up.

### Requirement Completion

Requirements complete **independently**. The player doesn't need to sustain all resources simultaneously. Once Ember's sustain timer finishes, it locks complete even if other resources are still running.

### UI States

| State | Condition | Visual |
|-------|-----------|--------|
| **Advancing** | Rate >= threshold | Green bar filling, rate in green |
| **Paused** | Rate < threshold | Bar frozen, pulses amber, rate in yellow |

Example line:
```
Ember:  ████████░░░░░░░░  15:00/30:00   52/hr (need 25/hr) ✓
Echo:   ████████████░░░░  22:00/30:00   11/hr (need 15/hr) ⏸
```

### Persistence

Save only `sustained_secs` per requirement. On load, restart the rolling window from empty (60-second ramp-up is negligible). Offline: use configured production rates and simulate normally.

## Required Fix: Buffer Overflow

Extractors produce into a 200-unit buffer. When full, production halts — silently breaking sustained rate patterns.

**Fix:** Auto-drain excess. When buffer hits capacity, excess production is discarded. The extractor keeps producing at full rate. The buffer exists as a reservoir for refineries, not as a production gate.

## Tier Gates (shifted for 28 patterns)

| Tier | Gate | What it unlocks |
|------|------|----------------|
| T1 | 1 pattern complete | Base x Base recipes |
| T2 | 8 patterns complete | Confluence x Base recipes |
| T3 | 15 patterns complete | Confluence x Confluence recipes (Woven Reality) |

## The 28-Pattern Progression

### Teaching Arc — 3 days (72 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 1 | First Thread | Ember 25/hr | 2 hr |
| 2 | Still Waters | Silence 25/hr | 2 hr |
| 3 | Echoing Halls | Memory 25/hr | 4 hr |
| 4 | Harmonic Pulse | Resonance 25/hr | 4 hr |
| 5 | Mirror and Void | Reflection 30/hr, VoidEssence 30/hr | 6 hr |
| 6 | Full Circle | All 6 base @ 20/hr | 10 hr |
| 7 | The Catalyst | CondensedEmber 8/hr | 16 hr |
| 8 | Echo of Flame | EmberEcho 8/hr | 28 hr |

### Mastery Arc — 10 days (236 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 9 | Forged in Fire | ForgedLight 15/hr | 16 hr |
| 10 | Glass Resonance | EchoGlass 15/hr | 16 hr |
| 11 | The Unsung | StillbornSong 15/hr | 24 hr |
| 12 | Void Distillation | PurifiedVoid 10/hr | 24 hr |
| 13 | Crossed Streams | ForgedLight 12/hr, EchoGlass 12/hr | 24 hr |
| 14 | The Asymmetry | ForgedLight 25/hr, StillbornSong 8/hr | 36 hr |
| 15 | Pressure Test | CondensedEmber 15/hr, EmberEcho 10/hr, PurifiedVoid 10/hr | 36 hr |
| 16 | Three Confluences | ForgedLight 18/hr, EchoGlass 18/hr, StillbornSong 18/hr | 60 hr |

### Endgame Arc — 22 days (534 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 17 | The Amplifier | ForgedLight 35/hr | 18 hr |
| 18 | Purified Cascade | PurifiedVoid 20/hr, ForgedLight 20/hr | 24 hr |
| 19 | Resonance Cascade | Resonance 150/hr, StillbornSong 25/hr | 24 hr |
| 20 | First Weave | WovenReality 5/hr | 30 hr |
| 21 | The Unraveling | WovenReality 15/hr, PurifiedVoid 15/hr | 36 hr |
| 22 | Grand Harmony | All 6 base @100/hr, all 3 confluence @30/hr | 36 hr |
| 23 | The Knot | ForgedLight 25/hr, PurifiedVoid 15/hr, CondensedEmber 12/hr | 36 hr |
| 24 | Strange Alchemy | ForgedLight 30/hr, EchoGlass 30/hr, StillbornSong 30/hr, Ember 80/hr, VoidEssence 80/hr | 42 hr |
| 25 | Refined Purpose | PurifiedVoid 30/hr, ForgedLight 25/hr | 48 hr |
| 26 | The Flood | WovenReality 35/hr | 48 hr |
| 27 | Everything Flows | All 13 resources at moderate rates | 72 hr |
| 28 | Mended Loom | WovenReality 20/hr, confluences @40/hr, Ember/Silence/Resonance @80/hr | 120 hr |

### Duration Summary

| Arc | Days | Range | Longest |
|-----|------|-------|---------|
| Teaching | 3 | 2 hr → 28 hr | Echo of Flame |
| Mastery | 10 | 16 hr → 60 hr | Three Confluences |
| Endgame | 22 | 18 hr → 120 hr | Mended Loom |
| **Total** | **35 days** | | |

### Resource Coverage

- All 13 resources featured at least once
- PurifiedVoid: 4 appearances (#12, #15, #18, #21)
- EchoGlass: solo spotlight (#10)
- Every base resource introduced individually before Full Circle (#6)
- WovenReality: 4 appearances (#20, #21, #26, #28)

### Mechanical Challenge Types

| Challenge | Patterns |
|-----------|---------|
| Raw throughput | #17 The Amplifier, #19 Resonance Cascade |
| Multi-tier chains | #20 First Weave, #21 The Unraveling |
| Full network | #22 Grand Harmony, #27 Everything Flows |
| Source contention | #23 The Knot, #15 Pressure Test |
| Recipe exploration | #24 Strange Alchemy |
| T2 depth | #25 Refined Purpose |
| Vertical scaling | #26 The Flood |
| Ultimate endurance | #28 Mended Loom |

## Narrative

### Discovery Text
> Beyond the Gateway, in a chamber older than memory, you find it — a vast mechanism of thread and light, broken and silent. The Loom of Worlds. Its spindles are dark, its weave unraveled. But as you draw near, something stirs. It has been waiting.

### Mastery Arc Opening
> The Loom no longer resists your touch. Its threads respond, its shuttle awaits. But comprehension is not mastery — now it demands not drops, but rivers. Sustain what you have learned. Feed it without faltering, and the weave will deepen beyond what teaching alone could reach.

### Endgame Arc Opening
> The Mastery Arc is complete. The Loom stirs — not with memory now, but with hunger. It remembers what it was, and demands you prove you can sustain what it will become. The final patterns require not moments of brilliance, but days of unwavering flow.

### Completion Text
> The Loom is whole. Across five days of unbroken flow, you wove what was shattered back into coherence. The hum beneath the world is steady now — not because it was fixed, but because you learned to sustain it. The Gateway dims. The work is done.

### Pattern Flavor Text

#### Teaching Arc

| # | Name | Flavor Text |
|---|------|------------|
| 1 | First Thread | A single ember, held steady, and the Loom stirs from its long silence. It drinks the warmth like parched earth drinks rain. |
| 2 | Still Waters | The Loom remembers stillness before fire. Feed it silence now — a sustained hush, patient and unbroken. |
| 3 | Echoing Halls | Hour after hour, memory flows into the weave. The Loom recalls what it was, one slow thread at a time. |
| 4 | Harmonic Pulse | A steady hum, sustained without faltering. The Loom tests whether its ancient frame can still hold a resonant frequency. |
| 5 | Mirror and Void | Form requires emptiness to fill. Feed the Loom both shape and absence together, and watch structure take root in nothing. |
| 6 | Full Circle | Six forces flow as one — ember, silence, memory, resonance, reflection, void. The Loom tastes the full spectrum for the first time in eons. |
| 7 | The Catalyst | Raw heat alone no longer suffices. The Loom demands ember compressed into dense purpose — a refinery's slow, steady yield. |
| 8 | Echo of Flame | Not fire itself but its afterimage — the memory of heat, distilled drop by drop. The Loom teaches you that recipe and rhythm both matter. |

#### Mastery Arc

| # | Name | Flavor Text |
|---|------|------------|
| 9 | Forged in Fire | Where ember meets void, light is born from contradiction. The Loom drinks this paradox through the night, steady as a forge that must not cool. |
| 10 | Glass Resonance | Memory poured through silence becomes glass that remembers. The Loom turns each reflection inward, weaving sixteen hours of frozen echoes into its frame. |
| 11 | The Unsung | A song caught between silence and resonance, never born yet always present. The silence that shapes it also feeds the glass — the first thread you must learn to share. |
| 12 | Void Distillation | To distill absence is to hold nothing so carefully it becomes substance. The Loom asks for pure potential, drawn slow and steady from the emptiness between things. |
| 13 | Crossed Streams | Light forged from fire, glass shaped from memory — both streams flowing at once. The Loom weaves with both hands now, and neither may falter. |
| 14 | The Asymmetry | The Loom demands a river of forged light but only a trickle of unsung sound. Unequal hungers require unequal commitment — one furnace will not suffice. |
| 15 | Pressure Test | Three streams drawn from shared roots, each pulling at the same deep wells of ember and void. The network strains. The Loom does not care — it drinks regardless. |
| 16 | Three Confluences | Light, glass, and silence-song sustained together for days without interruption. Every source is contested, every stream must hold. This is the full symphony the Loom was waiting to hear. |

#### Endgame Arc

| # | Name | Flavor Text |
|---|------|------------|
| 17 | The Amplifier | No single forge burns bright enough. The Loom demands a light that only parallel flames can cast. |
| 18 | Purified Cascade | Light must feed the void and still shine. Split the stream without dimming either side. |
| 19 | Resonance Cascade | The Forge screams for upgrades while Song drinks from the same well. Widen the source or drown in contention. |
| 20 | First Weave | A trickle of Woven Reality emerges from a chain three tiers deep. You are weaving existence itself now. |
| 21 | The Unraveling | Reality and void flow side by side, drawing from the same tiers. What feeds one starves the other without perfect coordination. |
| 22 | Grand Harmony | Nine rivers at once. Every extractor upgraded, every confluence sustained, every flow unbroken. The Loom hums on all frequencies. |
| 23 | The Knot | Three refineries. One Ember source. The Spindle cannot serve all masters — unless you untangle what seemed inseparable. |
| 24 | Strange Alchemy | Canonical recipes would starve the base flows the Loom also demands. Find stranger paths, or watch everything collapse. |
| 25 | Refined Purpose | Raw light is no longer enough. The Loom wants what only layered transmutation can provide. Build the chain deep. |
| 26 | The Flood | Parallel chains, eight refineries wide, all converging on a single thread. Reality pours forth like a river finding the sea. |
| 27 | Everything Flows | Thirteen resources. Seventy-two hours. Every tier, every recipe, every extractor singing in unison. This is the network you were always building toward. |
| 28 | Mended Loom | Five days. Every confluence roaring, every base resource at full draw, Woven Reality streaming into the ancient framework without pause. The Loom does not flicker. It does not falter. Thread by thread, the world holds. |
