---
name: wiki-audit
description: Multi-agent wiki audit — finds stale numbers, missing systems, and broken links in the player-facing wiki. Use when wiki is stale, after game changes, or before releases.
---

<!-- meta-audit scope-universe: quest.wiki/*.md -->

# Audit Player-Facing Wiki

Multi-agent audit of the GitHub wiki (`quest.wiki/` submodule). Finds stale content, missing systems, and broken links, then auto-fixes safe patterns.

## When to Use

- After landing new game systems, challenges, or mechanics
- When balance constants change (discovery rates, rewards, costs, XP curves)
- New zones, rooms, achievements, or equipment added
- Before a release

**For developer docs:** Use the `doc-audit` skill instead.

## Initialize Wiki Submodule

```bash
# Check status (- prefix = not initialized)
git submodule status quest.wiki

# Initialize if needed
git submodule update --init quest.wiki
```

## Phase 1: Parallel Audit (4 Agents, Read-Only)

Spawn 4 Explore agents simultaneously. Each agent cross-references wiki pages against source code to find discrepancies.

**Agent 1 — Core Gameplay**

Scope: `quest.wiki/Combat.md`, `quest.wiki/Zones-and-Progression.md`, `quest.wiki/Prestige.md`, `quest.wiki/Equipment.md`

Cross-reference against: `src/combat/`, `src/zones/`, `src/character/`, `src/items/`, `src/core/constants.rs`

**Agent 2 — Discovery Systems**

Scope: `quest.wiki/Dungeons.md`, `quest.wiki/Fishing.md`, `quest.wiki/Haven.md`, `quest.wiki/Soulforge.md`

Cross-reference against: `src/dungeon/`, `src/fishing/`, `src/haven/`, `src/enhancement/`

**Agent 3 — Late-Game Systems**

Scope: `quest.wiki/Challenges.md`, `quest.wiki/The-Deep.md`, `quest.wiki/Stormglass.md`, `quest.wiki/Achievements.md`

Cross-reference against: `src/challenges/`, `src/deep/`, `src/stormglass/`, `src/achievements/`

**Agent 4 — Guides & Meta**

Scope: `quest.wiki/Home.md`, `quest.wiki/Getting-Started.md`, `quest.wiki/Strategy-Guide.md`, `quest.wiki/Stormbreaker-Path.md`, `quest.wiki/Controls-and-UI.md`

Cross-reference against: `src/core/constants.rs`, `src/input/`, `src/ascension/`, `src/loom/`

### Anti-Patterns

Each agent searches for:

| Pattern | Severity | Example | Fix |
|---------|----------|---------|-----|
| Missing game system | HIGH | Loom not documented in wiki | Create or add section |
| Stale numbers/constants | HIGH | Wiki says "15% drop rate" but code says 20% | Update to match code |
| Missing zone/achievement/item | MEDIUM | New zones added but not in wiki | Add entries |
| Broken cross-links | MEDIUM | `[[Page Name]]` points to renamed page | Fix link |
| Stale strategy advice | LOW | Guide recommends outdated build order | Update advice |

Each agent produces a ranked report: page, pattern, severity (HIGH/MEDIUM/LOW), current value vs correct value, whether auto-fixable.

## Phase 2: Fix (Sequential)

Spawn fix agents based on audit findings.

### Auto-fix (no user approval needed)

- Update constant values to match source code
- Fix broken cross-links
- Add missing zones/achievements/items to existing tables

### Flag for user review

- Creating new wiki pages from scratch
- Removing documented sections
- Changing strategy/gameplay advice

### Wiki Update Rules

- Player-facing tone: friendly, engaging, practical tips alongside mechanics
- Use `[[Page Name]]` for cross-references
- Data tables for numbers-heavy content
- Translate code concepts to player language ("prestige" not "reset")
- Every page: "See Also" or "Related Pages" section

## Phase 3: Verify

1. All wiki pages match source constants
2. No broken cross-links
3. Report summary of: findings, auto-fixes applied, items flagged for review

### Push Wiki Changes

```bash
# Push wiki changes
cd quest.wiki && git add -A && git commit -m "docs: audit updates" && git push origin master && cd ..

# Update submodule pointer in main repo
git add quest.wiki
git commit -m "docs: update quest.wiki submodule pointer"
```

## Log This Run

After verification passes, record this run for `meta-audit`:

1. Build a JSON summary: date (YYYY-MM-DD), current commit SHA (`git rev-parse HEAD`), PR
   URL, agent count, the scope actually covered, and every finding (location, claim,
   correct value, severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 4,
     "scope": ["Combat.md", "Equipment.md"],
     "findings": [
       {
         "location": "Combat.md:42",
         "claim": "...",
         "correct_value": "...",
         "severity": "HIGH",
         "category": "stale-constant",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh wiki-audit /tmp/wiki-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh wiki-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `wiki-audit` as a follow-up
   after this run's own PR is up. If it prints `SKIP: n/5`, nothing further to do.

## Output

Report the PR URL and final status when done (use `/ship` skill).
