# The Deep — Balance Design

Concrete numbers for layer progression, economy, mercenary stats, and reward scaling. All values are tuned to support the walkthroughs in issue #362 and align with Quest's existing balance constants in `src/core/constants.rs`.

---

## 1. Layer Difficulty Curves

Each layer has a **Power threshold** — the minimum total squad Power for a comfortable Breakthrough. Squads below the threshold risk partial success or failure. Squads above it gain faster completion and better outcomes.

### Power Thresholds by Layer

| Layer | Tier | Power Threshold (Breakthrough) | Expedition Min | Recon Min | Supply Run Min |
|-------|------|-------------------------------|----------------|-----------|----------------|
| 1 | Shallows | 25 | 20 | 15 | 10 |
| 2 | Shallows | 40 | 30 | 20 | 15 |
| 3 | Shallows | 55 | 40 | 30 | 20 |
| 4 | Warrens | 75 | 55 | 40 | 25 |
| 5 | Warrens | 95 | 70 | 50 | 30 |
| 6 | Warrens | 115 | 85 | 60 | 40 |
| 7 | Warrens | 130 | 100 | 75 | 50 |
| 8 | Hollows | 155 | 115 | 85 | 55 |
| 9 | Hollows | 180 | 135 | 100 | 65 |
| 10 | Hollows | 205 | 155 | 115 | 75 |
| 11 | Hollows | 230 | 175 | 130 | 85 |
| 12 | Hollows | 260 | 195 | 145 | 95 |
| 13 | Sunken Reach | 295 | 220 | 165 | 110 |
| 14 | Sunken Reach | 330 | 250 | 185 | 125 |
| 15 | Sunken Reach | 370 | 280 | 210 | 140 |
| 16 | Sunken Reach | 410 | 310 | 230 | 155 |
| 17 | Sunken Reach | 450 | 340 | 255 | 170 |
| 18 | Sunken Reach | 495 | 370 | 280 | 185 |
| 19 | Abyss | 545 | 410 | 310 | 205 |
| 20 | Abyss | 600 | 450 | 340 | 225 |
| 21 | Abyss | 660 | 495 | 370 | 250 |
| 22 | Abyss | 720 | 540 | 405 | 275 |
| 23 | Abyss | 785 | 590 | 440 | 300 |
| 24 | Abyss | 855 | 640 | 480 | 325 |
| 25 | Abyss | 930 | 700 | 525 | 350 |
| 26+ | Void | 930 + 80*(L-25) | 700 + 60*(L-25) | 525 + 45*(L-25) | 350 + 30*(L-25) |

**Scaling rationale**: Shallows layers scale by ~15 Power per layer. Warrens by ~18. Hollows by ~25. Sunken Reach by ~33. Abyss by ~55. Void scales linearly at +80/layer (infinite endgame wall, analogous to Zone 11).

---

## 2. Mission Durations

Base durations before any modifiers (infrastructure, familiarity). All values in hours of real wall-clock time.

### Base Durations by Mission Type and Tier

| Layer Tier | Supply Run | Recon | Expedition | Breakthrough | Construction |
|------------|-----------|-------|------------|--------------|--------------|
| Shallows (1-3) | 2.0h | 4.0h | 8.0h | 18.0h | 4.0h |
| Warrens (4-7) | 2.5h | 5.0h | 10.0h | 20.0h | 5.0h |
| Hollows (8-12) | 3.0h | 6.0h | 12.0h | 22.0h | 6.0h |
| Sunken Reach (13-18) | 3.5h | 7.0h | 14.0h | 24.0h | 7.0h |
| Abyss (19-25) | 4.0h | 8.0h | 16.0h | 24.0h | 8.0h |
| Void (26+) | 4.0h | 8.0h | 16.0h | 24.0h | 8.0h |

### Duration Modifiers (multiplicative, stacking)

| Source | Reduction | Notes |
|--------|-----------|-------|
| Outpost infrastructure | -25% | Per-layer, permanent |
| Familiarity 25-49% (Mapped) | -10% | |
| Familiarity 50-74% (Familiar) | -20% | |
| Familiarity 75-100% (Mastered) | -30% | |
| Saboteur in squad | -10% to -15% | -10% base, -15% at Lv10+ |
| Overpowered squad (>150% threshold) | -10% | Cap at -10% |

**Example**: Supply Run on Layer 4 with Outpost + 75% Familiarity + Saboteur Lv10:
- Base: 2.5h
- After Outpost: 2.5 * 0.75 = 1.875h
- After Familiarity: 1.875 * 0.70 = 1.3125h
- After Saboteur: 1.3125 * 0.85 = 1.116h (~1h 7m)

**Minimum duration floor**: No mission can drop below 30 minutes. This prevents degenerate speed loops.

---

## 3. Warband Marks Economy

### Earning Rates by Layer and Mission Type

Mark rewards scale by layer. Formula: `base_marks * (1 + 0.08 * (layer - 1))` rounded to nearest 5.

| Layer | Supply Run | Recon | Expedition | Breakthrough |
|-------|-----------|-------|------------|--------------|
| 1 | 35 | 50 | 130 | 280 |
| 2 | 40 | 55 | 140 | 300 |
| 3 | 40 | 60 | 155 | 320 |
| 4 | 45 | 65 | 170 | 345 |
| 5 | 50 | 70 | 185 | 370 |
| 6 | 55 | 80 | 200 | 395 |
| 7 | 60 | 85 | 215 | 420 |
| 8 | 65 | 95 | 235 | 450 |
| 9 | 70 | 100 | 255 | 480 |
| 10 | 75 | 110 | 275 | 510 |
| 11 | 80 | 115 | 295 | 540 |
| 12 | 90 | 125 | 315 | 570 |
| 13 | 95 | 135 | 340 | 600 |
| 14 | 100 | 145 | 365 | 635 |
| 15 | 110 | 155 | 390 | 670 |
| 16 | 115 | 165 | 415 | 705 |
| 17 | 125 | 175 | 440 | 740 |
| 18 | 130 | 185 | 470 | 780 |
| 19 | 140 | 200 | 500 | 820 |
| 20 | 150 | 210 | 530 | 860 |
| 21 | 160 | 225 | 560 | 900 |
| 22 | 170 | 235 | 590 | 940 |
| 23 | 180 | 250 | 625 | 985 |
| 24 | 190 | 265 | 660 | 1030 |
| 25 | 200 | 280 | 695 | 1075 |
| 26+ | 200 + 10*(L-25) | 280 + 15*(L-25) | 695 + 35*(L-25) | 1075 + 50*(L-25) |

**Reward variance**: Actual rewards have +/- 15% random variance. A Layer 1 Supply Run pays 30-42 Marks (center 35).

**Partial success**: 60% of full rewards.
**Failure**: 20% of full rewards.

### Earning Modifiers

| Source | Bonus | Notes |
|--------|-------|-------|
| Supply Cache infrastructure | +50% Marks | Per-layer supply runs only |
| Familiarity 75-100% | +15% Marks | Bonus yield on Mastered layers |
| Full Success | 100% | Standard |
| Partial Success | 60% | |
| Failure | 20% | |

### Spending Sinks

| Cost | Marks | Notes |
|------|-------|-------|
| Recruit merc (Common quality) | 30-50 | Random within range per candidate |
| Recruit merc (Uncommon quality) | 50-80 | Better base stats |
| Recruit merc (Rare quality) | 80-120 | Best stats, rarer archetypes |
| Launch Supply Run | 0 (free daily) or 15-25 | One free per day on any cleared layer |
| Launch Recon | 30-50 | Scales slightly with layer: 30 + layer |
| Launch Expedition | 80-150 | Scales: 80 + 3*layer |
| Launch Breakthrough | 150-350 | Scales: 150 + 8*layer |
| Build Outpost | 60 + 4*layer | Layer 1: 64, Layer 10: 100, Layer 20: 140 |
| Build Supply Cache | 80 + 5*layer | Layer 1: 85, Layer 10: 130, Layer 20: 180 |
| Build Watchtower | 70 + 4*layer | Layer 1: 74, Layer 10: 110, Layer 20: 150 |
| Build Bridge | 100 + 5*layer | Layer 1: 105, Layer 10: 150, Layer 20: 200 |
| Guild Rank 2 | 200 | One-time, persists |
| Guild Rank 3 | 500 | One-time, persists |
| Guild Rank 4 | 1,200 | One-time, persists |
| Guild Rank 5 | 3,000 | One-time, persists |

### Economy Flow Validation

**Day 1 (fresh P15, Generation 1):**
- Free Supply Run L1: +35 Marks (2h)
- Recon L1: -30, +50 Marks net +20 (4h)
- End of Day 1: ~55 Marks, 1 cleared layer

**Week 1 end (Generation 1):**
- ~480 Marks accumulated
- Layers 1-2 cleared, Layer 3 frontier
- 1-2 infrastructure buildings
- 4/5 mercs

**Mid-game (Generation 3, Rank 3, Layers 1-8 cleared):**
- Daily passive from supply circuit (2 slots): ~200-300 Marks/day
- Cost to push 1 frontier layer: ~250-400 Marks (Recon + Expedition + Breakthrough)
- Net progress: ~1 layer every 2-3 days

**Endgame (Generation 8+, Rank 5, Layers 1-22 cleared):**
- Daily passive from 4-slot supply circuit: ~600-1000 Marks/day
- Cost to push 1 frontier layer: ~600-900 Marks
- Net progress: ~1 layer every 1-2 days

---

## 4. Mercenary Stat Curves

### Base Stats by Archetype (Level 1)

Each merc has 3 stats: Power, Resilience, Expertise. Base stats at Level 1 depend on archetype and guild rank quality tier.

#### Rank 1 (Freelancers) — Common Quality

| Archetype | Power | Resilience | Expertise | Total |
|-----------|-------|------------|-----------|-------|
| Vanguard | 12 | 14 | 8 | 34 |
| Scout | 8 | 10 | 14 | 32 |
| Arcanist | 10 | 8 | 14 | 32 |
| Medic | 6 | 12 | 12 | 30 |
| Saboteur | 9 | 9 | 14 | 32 |

#### Rank 2 (Sellswords) — Common/Uncommon Quality

Base stats increase by +2 per stat compared to Rank 1 pool averages. Uncommon recruits get an additional +2 to their primary stats.

| Quality | Power Range | Resilience Range | Expertise Range |
|---------|------------|------------------|-----------------|
| Common | +2 over Rank 1 bases | +2 | +2 |
| Uncommon | +4 to primary, +2 to others | +4 to primary, +2 to others | +4 to primary, +2 to others |

#### Rank 3 (Company) — Uncommon/Rare Quality

+4 per stat over Rank 1 bases. Rare recruits get +6 to primary stats.

#### Rank 4 (Battalion) — Rare Quality Standard

+6 per stat over Rank 1 bases. Rare recruits get +8 to primary stats.

#### Rank 5 (Legion) — Rare/Elite Quality

+8 per stat over Rank 1 bases. Elite recruits get +12 to primary stats.

**Summary: Level 1 Stats by Rank (Vanguard archetype as example)**

| Rank | Quality | Power | Resilience | Expertise |
|------|---------|-------|------------|-----------|
| 1 | Common | 12 | 14 | 8 |
| 2 | Common | 14 | 16 | 10 |
| 2 | Uncommon | 16 | 18 | 10 |
| 3 | Uncommon | 16 | 18 | 12 |
| 3 | Rare | 18 | 20 | 12 |
| 4 | Rare | 18 | 20 | 14 |
| 4 | Rare+ | 20 | 22 | 14 |
| 5 | Rare | 20 | 22 | 16 |
| 5 | Elite | 24 | 26 | 16 |

### Level Scaling (1-20)

Stats grow per level based on archetype weights. Growth is **not** linear — early levels grow fast, later levels taper.

**Per-level stat growth formula**: `stat_at_level = base + growth_per_level * (level - 1)`

| Archetype | Power/Lvl | Resilience/Lvl | Expertise/Lvl |
|-----------|-----------|----------------|---------------|
| Vanguard | +4.0 | +3.5 | +2.0 |
| Scout | +3.0 | +3.0 | +3.5 |
| Arcanist | +3.5 | +2.0 | +4.0 |
| Medic | +2.0 | +3.5 | +3.0 |
| Saboteur | +3.0 | +2.5 | +4.0 |

**Example: Rank 1 Vanguard Level Progression**

| Level | Power | Resilience | Expertise | Total |
|-------|-------|------------|-----------|-------|
| 1 | 12 | 14 | 8 | 34 |
| 2 | 16 | 18 | 10 | 44 |
| 3 | 20 | 21 | 12 | 53 |
| 4 | 24 | 25 | 14 | 63 |
| 5 | 28 | 28 | 16 | 72 |
| 8 | 40 | 39 | 22 | 101 |
| 10 | 48 | 46 | 26 | 120 |
| 12 | 56 | 53 | 30 | 139 |
| 15 | 68 | 63 | 36 | 167 |
| 18 | 80 | 74 | 42 | 196 |
| 20 | 88 | 81 | 46 | 215 |

### Merc XP and Leveling

Mercs gain XP from completing missions. XP per mission scales with mission type and layer.

| Mission Type | Base XP | Layer Scaling |
|-------------|---------|---------------|
| Supply Run | 100 | +10 per layer |
| Recon | 200 | +20 per layer |
| Expedition | 400 | +40 per layer |
| Breakthrough | 800 | +80 per layer |
| Construction | 50 | Flat (no scaling) |

**XP to next level**: `200 * level^1.3` (same curve shape as main game, different scale)

| Level | XP Required | Cumulative XP |
|-------|------------|---------------|
| 1->2 | 200 | 200 |
| 2->3 | 492 | 692 |
| 3->4 | 851 | 1,543 |
| 4->5 | 1,262 | 2,805 |
| 5->6 | 1,716 | 4,521 |
| 8->9 | 3,436 | 15,074 |
| 10->11 | 4,689 | 24,226 |
| 14->15 | 8,281 | 56,610 |
| 18->19 | 12,583 | 104,258 |
| 19->20 | 13,869 | 118,127 |

**Leveling pace**: A merc running Supply Runs on Layer 5 (~150 XP per run, ~3h each) takes roughly:
- Level 1->5: ~6-7 supply runs (~20h)
- Level 5->10: ~20 supply runs (~60h, ~3 days)
- Level 10->15: ~50 supply runs (~150h, ~7 days)
- Level 15->20: ~100+ supply runs (~300h+, unlikely before prestige)

This ensures merc leveling is meaningful but not the primary bottleneck. Most mercs reach Level 8-12 in a typical generation. Level 15+ requires intentional investment. Level 20 is exceptional.

### Squad Power Calculation

Total squad Power = sum of individual merc Power stats.

**Comfortable margin**: Squad Power >= 110% of threshold. Good auto-resolve odds.
**Tight margin**: Squad Power 100-110% of threshold. 65-75% full success chance.
**Underpowered**: Squad Power < 100% of threshold. Risk of partial success or failure.

---

## 5. Familiarity System

### Familiarity Gain Per Mission

| Mission Type | Familiarity Gain | Notes |
|-------------|-----------------|-------|
| Supply Run | +5% | Slow but safe |
| Recon | +15% | Primary familiarity builder |
| Expedition | +10% | Secondary gain |
| Breakthrough | +15% | One-time per layer |
| Construction | +5% | Small bonus |

**Watchtower infrastructure**: Grants +25% Familiarity immediately on construction.

### Familiarity Thresholds

| Range | Status | Mission Duration | Auto-Resolve Quality | Mark Bonus |
|-------|--------|-----------------|---------------------|------------|
| 0-24% | Unknown | Base | Poor (65% safe option success) | None |
| 25-49% | Mapped | -10% | Fair (75%) | None |
| 50-74% | Familiar | -20% | Good (85%) | None |
| 75-100% | Mastered | -30% | Excellent (95%) | +15% |

### Familiarity Persistence

Familiarity persists across prestiges. It never decreases. This means:
- Generation 1: Layer 1 reaches ~70% Familiarity through normal play
- Generation 2: Layer 1 starts at 70%, reaches 85%+ with a few more missions
- Generation 3+: Layer 1 is at 95-100%, missions are lightning fast

**Cap**: 100%. Beyond 100% has no additional effect.

---

## 6. Infrastructure ROI

### Cost and Effect Summary

| Infrastructure | Cost Formula | Effect | Persists |
|---------------|-------------|--------|----------|
| Outpost | 60 + 4*layer | -25% mission duration this layer | Yes |
| Supply Cache | 80 + 5*layer | +50% Supply Run Marks this layer | Yes |
| Watchtower | 70 + 4*layer | +25 Familiarity, better auto-resolve | Yes |
| Bridge | 100 + 5*layer | -2h on missions transiting through this layer | Yes |

### Supply Cache ROI Analysis

The Supply Cache is the primary income-generating infrastructure. ROI = cost / extra marks per run.

| Layer | Cache Cost | Base Supply Run Marks | +50% Bonus | Extra/Run | Runs to Break Even |
|-------|-----------|----------------------|-----------|-----------|-------------------|
| 1 | 85 | 35 | 52 | 17 | 5.0 |
| 3 | 95 | 40 | 60 | 20 | 4.8 |
| 5 | 105 | 50 | 75 | 25 | 4.2 |
| 8 | 120 | 65 | 97 | 32 | 3.8 |
| 10 | 130 | 75 | 112 | 37 | 3.5 |
| 12 | 140 | 90 | 135 | 45 | 3.1 |
| 15 | 155 | 110 | 165 | 55 | 2.8 |
| 20 | 180 | 150 | 225 | 75 | 2.4 |

**Key insight**: Supply Caches pay for themselves in 3-5 runs (6-15 hours of wall time depending on layer and speed modifiers). After that, they generate pure profit forever. This is the core of the infrastructure ratchet.

### Outpost ROI Analysis

The Outpost saves time, not Marks directly. Value measured in time saved per mission.

| Layer | Outpost Cost | Supply Run Base Duration | -25% Savings | Missions to Save 1h |
|-------|-------------|-------------------------|-------------|---------------------|
| 1 | 64 | 2.0h | 0.5h | 2 |
| 5 | 80 | 2.5h | 0.625h | 1.6 |
| 10 | 100 | 3.0h | 0.75h | 1.3 |
| 15 | 120 | 3.5h | 0.875h | 1.1 |
| 20 | 140 | 4.0h | 1.0h | 1.0 |

**Practical value**: Time savings compound with Supply Cache. An Outpost + Supply Cache layer generates Marks per hour at ~2x the rate of a raw layer. Both together cost ~145-320 Marks depending on layer but generate permanent compounding returns.

### Optimal Build Orders

**Economy-first (recommended for new players):**
1. Supply Cache on Layer 1 (best ROI early)
2. Outpost on Layer 1 (speed up the earning)
3. Supply Cache on Layer 3-4 (second income source)
4. Push frontier to Layer 7 for Guild Rank 3
5. Supply Cache on highest cleared layer
6. Repeat: cache on best supply layer, push frontier

**Speed-first (for experienced players):**
1. Outpost on Layer 1 (fast runs)
2. Bridge on Layer 2 (shortcut for deep missions)
3. Push frontier aggressively
4. Backfill Supply Caches after reaching Rank 3

**Balance (hybrid):**
1. Supply Cache on Layer 1
2. Push to Layer 3
3. Outpost + Supply Cache on Layer 3
4. Push to Layer 7, buy Rank 3
5. Fill in Supply Caches on Layers 4-7
6. Use 2 slots to run supply + push simultaneously

---

## 7. Guild Rank Costs and Requirements

| Rank | Name | Mark Cost | Layer Requirement | Max Roster | Concurrent Missions | Recruit Quality |
|------|------|-----------|-------------------|------------|---------------------|-----------------|
| 1 | Freelancers | Free | Discovery | 5 | 1 | Common |
| 2 | Sellswords | 200 | Layer 3 cleared | 7 | 1 | Common + Uncommon |
| 3 | Company | 500 | Layer 7 cleared | 9 | 2 | Uncommon + Rare |
| 4 | Battalion | 1,200 | Layer 13 cleared | 12 | 3 | Rare |
| 5 | Legion | 3,000 | Layer 19 cleared | 15 | 4 | Rare + Elite |

### Rank Upgrade Timeline (expected)

| Rank | Earliest Generation | Typical Day in Generation | Cumulative Marks Earned |
|------|--------------------|--------------------------|-----------------------|
| 2 | Gen 1 | Day 5-8 | ~500 |
| 3 | Gen 1-2 | Day 12-18 | ~2,000 |
| 4 | Gen 3-4 | Day 8-14 (of that gen) | ~5,000 |
| 5 | Gen 5-6 | Day 10-15 (of that gen) | ~12,000 |

### Guild Rank Effects on Recruitment Pool

| Rank | Pool Size | Quality Distribution | Archetype Availability |
|------|-----------|---------------------|----------------------|
| 1 | 3 candidates | 100% Common | Vanguard, Scout, Medic only |
| 2 | 4 candidates | 60% Common, 40% Uncommon | + Arcanist |
| 3 | 4 candidates | 30% Common, 50% Uncommon, 20% Rare | + Saboteur |
| 4 | 5 candidates | 40% Uncommon, 50% Rare, 10% Elite | All archetypes |
| 5 | 5 candidates | 20% Uncommon, 50% Rare, 30% Elite | All archetypes |

**Daily pool refresh**: Pool refreshes every 24 hours (wall clock) or on prestige, whichever comes first. Unrecruited candidates are lost.

---

## 8. Injury and Loss Probability Curves

### Risk Levels by Mission Type

| Mission Type | Base Injury Chance | Base Loss Chance | Notes |
|-------------|-------------------|-----------------|-------|
| Supply Run (cleared) | 0% | 0% | Always safe |
| Recon (frontier) | 10% | 0% | Can injure, never lose |
| Expedition (frontier) | 20% | 2% | Moderate risk |
| Breakthrough (frontier) | 35% | 5% | Highest risk |
| Construction (cleared) | 0% | 0% | Always safe |

### Modifiers to Injury/Loss

| Factor | Injury Modifier | Loss Modifier | Notes |
|--------|----------------|---------------|-------|
| Underpowered squad (<100% threshold) | +15% | +5% | Significant penalty |
| Overpowered squad (>120% threshold) | -10% | -2% | Safety from strength |
| Medic in squad | -10% injury | Loss -> Injury downgrade | Medic prevents loss |
| Vanguard in squad | -5% all | -2% | Frontline protection |
| High Resilience (avg >50) | -5% | -1% | Experienced squads |
| Failed event choice | +10% | +3% | Cascading consequences |
| Partial Success outcome | +15% injury | +3% loss | Rough mission |
| Failure outcome | +25% injury | +8% loss | Very rough |

**Medic loss prevention**: When a merc would be lost, if a Medic is in the squad, the loss is downgraded to a severe injury (16h recovery) instead. The Medic ability triggers with probability `50% + Medic_Level * 2.5%` (max 100% at Level 20). This is the core reason Medics are valuable.

### Injury Duration

| Severity | Recovery Time | Trigger |
|----------|-------------|---------|
| Light | 4-8h | Standard injury roll |
| Moderate | 8-12h | Failed event + injury |
| Severe | 12-16h | Medic-prevented loss |

**Mid-mission injury**: A merc injured during a mission (from an event) operates at -20% Power for the remainder of the mission and then enters recovery for 8-12h after the mission completes.

---

## 9. Reward Scaling

### XP Rewards

XP from The Deep feeds into the main game's XP pool. Scaled to be meaningful but not dominant over the combat loop.

| Mission Type | Base XP | Layer Scaling | Layer 1 XP | Layer 10 XP | Layer 20 XP |
|-------------|---------|---------------|-----------|------------|------------|
| Supply Run | 150 | +20/layer | 170 | 350 | 550 |
| Recon | 300 | +35/layer | 335 | 650 | 1000 |
| Expedition | 600 | +60/layer | 660 | 1200 | 1800 |
| Breakthrough | 1200 | +120/layer | 1320 | 2400 | 3600 |

**Context**: The main combat loop awards 200-400 XP per kill (every ~4-5 seconds). A Layer 10 Expedition (12h) awards 1200 XP — equivalent to about 3-6 kills. The Deep's XP contribution is a nice bonus, not a replacement for combat. Its value is in the other rewards (Marks, items, Stormglass, PR fragments).

### Item Rewards

| Mission Type | Item Drop Chance | Max Rarity | Notes |
|-------------|-----------------|-----------|-------|
| Supply Run | 20% | Common (L1-7), Uncommon (L8+) | Low-quality drops |
| Recon | 30% | Uncommon | |
| Expedition | 60% | Rare (L1-12), Epic (L13+) | Primary item source |
| Breakthrough | 100% | Epic (L1-12), Legendary (L13+) | Guaranteed drop |

**Item level**: ilvl = layer * 10 (matching zone ilvl scaling: Zone 1 = ilvl 10, Layer 10 = ilvl 100).

**Abyssal equipment** (Layers 19+): 10% chance on Expedition, 25% chance on Breakthrough. These items have standard combat stats PLUS one Deep-specific affix:

| Abyssal Affix | Effect |
|--------------|--------|
| Expedition Haste | +5-15% mission speed (scales with ilvl) |
| Deep Harvest | +10-25% supply run yield |
| Abyssal Ward | +5-15% squad Resilience |
| Voidtouch | +5-15% squad Power |
| Cartographer's Insight | +5-10% Familiarity gain |

### Stormglass Rewards

Stormglass is earned from Expeditions and Breakthroughs only. Requires P15+ (same gate as Stormglass system).

| Mission Type | Stormglass | Layer Scaling |
|-------------|-----------|---------------|
| Supply Run | 0 | None |
| Recon | 0 | None |
| Expedition | 5 + floor(layer/3) | L1: 5, L10: 8, L20: 11 |
| Breakthrough | 10 + floor(layer/2) | L1: 10, L10: 15, L20: 20 |

### Prestige Rank Fragments

Breakthrough missions on Layers 8+ award fractional prestige ranks. This creates an alternate prestige income path.

| Layer Range | PR Fragment per Breakthrough | Notes |
|-------------|---------------------------|-------|
| 1-7 | 0 | Too shallow for PR |
| 8-12 | 0.25 PR | Modest contribution |
| 13-18 | 0.50 PR | Meaningful supplement |
| 19-25 | 0.75 PR | Significant alternative |
| 26+ | 1.00 PR | Full rank per breakthrough |

**Prestige rank fragments accumulate across Breakthroughs and are awarded as whole ranks when they reach integer values.** (e.g., four Layer 8-12 Breakthroughs = +1 PR)

---

## 10. Auto-Resolve Quality

When events are not manually resolved (player offline or 2h auto-resolve timer expires), the system picks the safest option. Auto-resolve quality determines how good the "safe" option is.

| Familiarity | Auto-Resolve Success Rate | Delay Penalty |
|-------------|--------------------------|---------------|
| 0-24% (Unknown) | 65% | +2h average |
| 25-49% (Mapped) | 75% | +1.5h average |
| 50-74% (Familiar) | 85% | +1h average |
| 75-100% (Mastered) | 95% | +0.5h average |

**Scout bonus**: +10% to auto-resolve success rate when a Scout is in the squad.
**Watchtower infrastructure**: +5% to auto-resolve success rate on that layer.

---

## 11. Discovery Constants

Following existing patterns from Haven and Soulforge:

```
DEEP_MIN_PRESTIGE_RANK: 15
DEEP_DISCOVERY_BASE_CHANCE: 0.000014  // Per tick, same as Haven/Soulforge
DEEP_DISCOVERY_RANK_BONUS: 0.000007  // Per rank above 15
```

At P15: `0.000014` per tick = ~71,400 ticks = ~7,140 seconds = ~2 hours average.
At P20: `0.000014 + 5*0.000007 = 0.000049` per tick = ~1.7 hours average.

---

## 12. Prestige Reset Behavior

### What Resets
- All mercenaries disbanded
- All active missions cancelled (partial rewards at 50% of earned-so-far)
- Warband Marks set to 0
- Merc levels reset (new recruits start at Lv1)

### What Persists
- Guild Rank (and all its benefits)
- All cleared layers (permanently cleared)
- All built infrastructure (2 slots per layer)
- All Familiarity levels (never decrease)

### Generation Startup Package
- 3 free starter mercs (quality based on current Guild Rank)
- 1 free Supply Run immediately available
- All cleared layers accessible for missions immediately

---

## 13. Balance Validation Scenarios

### Scenario A: Fresh Discovery (P15, Gen 1)
- Start: 0 Marks, 3 free mercs (Rank 1 Common), 0 layers cleared
- Day 1: Clear Layer 1, earn ~90 Marks
- Day 3: Layer 2 cleared, ~300 Marks, first infrastructure on L1
- Day 7: Layer 3 cleared, ~500 Marks, Guild Rank 2 purchased, 5-6 mercs
- Day 14: Layer 5-6, ~1000 cumulative Marks earned, approaching Rank 3

### Scenario B: Second Generation (P17, Gen 2, Rank 2, L1-5 cleared)
- Start: 0 Marks, 3 free mercs (Rank 2 quality), Layers 1-5 cleared with infrastructure
- Day 1: 200+ Marks from optimized supply runs, recruit 2 mercs
- Day 3: Back at Layer 5 frontier, 500 Marks
- Day 7: Layer 7 cleared, buy Rank 3 (500 Marks), 2 mission slots unlocked

### Scenario C: Endgame Push (P28, Gen 8, Rank 5, L1-21 cleared)
- Start: 0 Marks, 3 free mercs (Rank 5 Elite quality), deep infrastructure network
- Day 1: 500+ Marks from optimized 2-slot supply circuit
- Day 2: 9+ mercs recruited, 4 slots running, frontier L22 recon begins
- Day 5: Layer 22 Breakthrough attempted, back to pushing frontier
- Steady state: ~1 new layer every 2 days

### Key Balance Checks

1. **Supply Cache never outpaces frontier rewards**: Supply Run income on cleared layers should always be less than Breakthrough/Expedition rewards on frontier. This ensures pushing the frontier remains attractive.

2. **Guild Rank 3 is achievable in Gen 1-2**: The 2nd mission slot is the most impactful upgrade. Players should reach it within 2-3 weeks of real play.

3. **Losing a high-level merc stings but doesn't brick progress**: Even losing a Lv15+ merc, the player can recruit from a Rank 3+ pool with good base stats. Recovery time: 2-3 days to level a replacement to usefulness.

4. **Infrastructure investment always pays off within the same generation**: Supply Cache ROI is 3-5 runs. Even if you prestige soon after building, the infrastructure persists and pays dividends in every future generation.

5. **Marks-per-hour scales sub-linearly with layer depth**: Deeper layers pay more Marks but take longer. Marks/hour stays in the ~15-45 range across all layers, preventing any single layer from being strictly dominant for farming.

| Layer | Supply Run Marks | Duration (no mods) | Marks/Hour |
|-------|-----------------|-------------------|------------|
| 1 | 35 | 2.0h | 17.5 |
| 5 | 50 | 2.5h | 20.0 |
| 10 | 75 | 3.0h | 25.0 |
| 15 | 110 | 3.5h | 31.4 |
| 20 | 150 | 4.0h | 37.5 |
| 25 | 200 | 4.0h | 50.0 |

With Supply Cache (+50%) and infrastructure modifiers, optimized layers can reach 50-60 Marks/hour. This is intentional — it rewards infrastructure investment without breaking the economy.
