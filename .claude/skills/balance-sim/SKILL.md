---
name: balance-sim
description: Multi-agent game balance simulator — runs the headless simulator across strategies and seeds to test Z50/Ascension X progression, then produces a prioritized balance report. Use when you want to check game balance, after landing combat/zone/loom changes, before releases, or when asked to "run the simulator", "check balance", or "can we beat Z50".
---

# Balance Simulator

Run the headless game simulator across multiple strategy profiles and seeds in parallel, then analyze results to assess whether the game can be cleared (Z50 + Ascension X) and identify progression bottlenecks.

## Overview

The simulator (`src/bin/simulator/`) exercises real game logic via `game_tick_with_context()`. Interactive systems (challenges, enhancement, sigils, Deep layers) are injected by strategy profiles. Loom production uses real extractors/shuttles/pattern sustain.

All runs start from **P0** (prestige rank 0) with `--stormbreaker` enabled (unlocks Zone 10 boss). This tests full organic progression.

**Key game constants for reference:**
- Ascension costs: I-VI = [35, 65, 120, 200, 325, 500] PR; VII-X = [1500, 4000, 8000, 15000] PR
- Ascension gates: I-VI need Deep Layers [3, 7, 12, 18, 25, 30]; VII-X need [8, 16, 22, 28] Woven Patterns
- Fracture zone scaling: 1.6x per zone from Z11 base
- Loom zone scaling: 1.25x per zone from Z30 base
- Loom zone unlock: 4p→Z31-34, 8p/AscVII→Z35-38, 16p/AscVIII→Z39-42, 22p/AscIX→Z43-46, 28p/AscX→Z47-50

## Phase 1: Build

Build the release binary once before spawning agents. Run from the repo root (or the worktree root if on a worktree):

```bash
cargo build --release --bin simulator
```

If the build fails, fix the issue before proceeding.

## Phase 2: Parallel Simulation Runs

Spawn **9 agents** in a single message (3 strategies × 3 seeds), each with `mode: "bypassPermissions"`. Do NOT use worktree isolation — all agents share the same release binary.

Each agent runs the simulator, writes a Python analysis script, and returns structured JSON.

### Agent Configuration Matrix

| Agent | Strategy | Seed | Ticks | Simulated Time |
|-------|----------|------|-------|----------------|
| 1 | casual | 42 | 360000000 | 10,000 hrs |
| 2 | casual | 123 | 360000000 | 10,000 hrs |
| 3 | casual | 7 | 360000000 | 10,000 hrs |
| 4 | optimal | 42 | 180000000 | 5,000 hrs |
| 5 | optimal | 123 | 180000000 | 5,000 hrs |
| 6 | optimal | 7 | 180000000 | 5,000 hrs |
| 7 | speedrun | 42 | 108000000 | 3,000 hrs |
| 8 | speedrun | 123 | 108000000 | 3,000 hrs |
| 9 | speedrun | 7 | 108000000 | 3,000 hrs |

Simulated time = ticks / 10 (ticks per second) / 3600 (seconds per hour). This is exact — do not recalculate.

### Agent Prompt Template

Each agent receives this prompt (substitute `{strategy}`, `{seed}`, `{ticks}`, `{sim_hours}`, `{cwd}`):

> You are analyzing a Quest balance simulation run. Work in `{cwd}`.
>
> **Step 1: Run the simulator**
> ```bash
> cd {cwd} && cargo run --release --bin simulator -- \
>   --strategy {strategy} --seed {seed} --stormbreaker \
>   --ticks {ticks} --csv /tmp/balance-sim-{strategy}-{seed}.csv --quiet
> ```
> The simulation time is exactly **{sim_hours} hours** ({ticks} ticks / 10 tps / 3600).
>
> **Step 2: Analyze with Python**
> Write and run a Python script that reads `/tmp/balance-sim-{strategy}-{seed}.csv` and outputs JSON to stdout. The script must:
>
> 1. Compute `sim_hours` as `{sim_hours}` (the exact value, not derived from CSV).
> 2. Read the last CSV row for final state: `zone_id`, `ascension_level`, `level` (character level), `total_kills`, `total_deaths`, `pr_earned`, `pr_spent`.
> 3. Scan for ascension milestones: for each ascension_level transition (0→1, 1→2, ...), record `{"level": N, "hours": tick/36000, "zone_at": zone_id}` from the first row at that level.
> 4. Scan for zone milestones: for zones [11, 20, 30, 34, 38, 42, 46, 50], find the first row where `zone_id >= Z` and record `{"zone": Z, "hours": tick/36000, "ascension_at": ascension_level}`. Skip zones never reached.
> 5. Detect walls: for each zone the hero visits, compute total time spent there. If time > 20% of {sim_hours}, it's a wall. Record `{"zone": Z, "hours_stuck": H, "pct_of_sim": P}`.
> 6. Output a single JSON object to stdout with keys: `strategy`, `seed`, `ticks`, `sim_hours`, `final_zone`, `final_ascension`, `final_level`, `total_kills`, `total_deaths`, `pr_earned`, `pr_spent`, `reached_z50` (bool), `reached_asc_x` (bool), `ascension_milestones`, `zone_milestones`, `walls`.
>
> **Step 3: Return the JSON**
> Run the script and return ONLY the JSON output. No commentary, no markdown fences, no explanation — just the raw JSON object.

## Phase 3: Compile Report

After all 9 agents complete, parse each agent's JSON result. If an agent returned commentary around the JSON, extract just the JSON object (look for `{` to `}`).

### Cross-Reference with Game Constants

Before writing recommendations, read the strategy configuration in `src/bin/simulator/strategy.rs` to check for simulator-side issues:
- Does each strategy's `deep_max_layer()` reach 30? (Required for Asc VI)
- Does each strategy's `challenge_interval_ticks()` and reward match expectations?
- Does `loom_pr_threshold()` allow Loom to start early enough?

Separate **simulator bugs** (wrong strategy config) from **game balance issues** (constants that make progression too slow/fast).

### Report Structure

Generate this report in markdown:

```markdown
# Balance Simulation Report — {date}

## Executive Summary
- Did any strategy clear Z50? Which ones, at what hour?
- Did all strategies reach Asc X? At what hour range?
- What is the furthest zone reached per strategy (median across seeds)?
- One-sentence verdict: is the game beatable from P0?

## Progression Timeline

### By Strategy
Median hours across 3 seeds for key milestones:

| Milestone | Casual | Optimal | Speedrun |
|-----------|--------|---------|----------|
| Asc I | ... | ... | ... |
| Asc II | ... | ... | ... |
| Asc III | ... | ... | ... |
| Asc IV | ... | ... | ... |
| Asc V | ... | ... | ... |
| Asc VI | ... | ... | ... |
| Asc VII | ... | ... | ... |
| Asc VIII | ... | ... | ... |
| Asc IX | ... | ... | ... |
| Asc X | ... | ... | ... |
| Zone 11 | ... | ... | ... |
| Zone 20 | ... | ... | ... |
| Zone 30 | ... | ... | ... |
| Zone 34 | ... | ... | ... |
| Zone 38 | ... | ... | ... |
| Zone 42 | ... | ... | ... |
| Zone 46 | ... | ... | ... |
| Zone 50 | ... | ... | ... |

Use "—" for milestones never reached. Include all ascension levels I-X, not just selected ones.

### Variance Analysis
Note milestones where seed range exceeds 25% of the median. Low variance = deterministic, high variance = RNG-sensitive.

## Progression Walls
Group walls by zone across all 9 runs:
- How many of 9 runs hit this wall
- Median time stuck (hours and % of sim)
- What resolved it (or "unresolved" if still stuck at sim end)
- Root cause: reference specific game constants (e.g., "Asc VI requires Deep L30, fracture scaling is 1.6x/zone")

## Balance Assessment

| Phase | Zones | Rating (1-5) | Notes |
|-------|-------|--------------|-------|
| Early Game | 1-10 | ... | ... |
| Fracture Early | 11-20 | ... | ... |
| Fracture Late | 21-30 | ... | ... |
| Loom Entry | 31-34 | ... | ... |
| Loom Mid | 35-42 | ... | ... |
| Loom Late | 43-46 | ... | ... |
| Loom Final | 47-50 | ... | ... |

Rating scale: 1=broken/impossible, 2=too hard, 3=well-balanced, 4=too easy, 5=trivially easy

## Simulator Issues
List any problems found in the simulator strategy config (not game balance):
- Strategy config bugs (wrong caps, missing injections)
- Missing CSV columns that would help analysis
- Suggestions for simulator improvements

## Recommendations

Ordered by priority (P0 first):

### P0 — [Title]
- **Problem**: ...
- **Data**: which runs, what numbers
- **Suggested Fix**: specific constant changes with old→new values
- **Expected Impact**: ...
- **Files**: ...

(repeat for P1, P2)
```

### Writing Guidelines

- Use **median** values across seeds, not means
- If a milestone was never reached by any seed for a strategy, mark as "—"
- Focus recommendations on the **first unresolved wall** per strategy — that's where players actually get stuck
- Reference specific constants from `CLAUDE.md` and `src/bin/simulator/strategy.rs`
- Be specific: "reduce Asc X cost from 15000 to 5000 PR" not "reduce ascension costs"
- Distinguish simulator bugs from game balance issues — they need different fixes

## Phase 4: Present Results

Print the full report to the user. If no strategy cleared Z50, highlight this prominently in the executive summary.
