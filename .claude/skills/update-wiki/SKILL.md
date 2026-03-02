---
name: update-wiki
description: Use when the player-facing wiki (quest.wiki/) needs updating after game changes, new systems, rebalancing, or before releases. Use when wiki pages have stale numbers, missing systems, or need new content.
---

# Update Player-Facing Wiki

Audit and update the GitHub wiki (`quest.wiki/` submodule) to match the current game.

**For developer docs:** Use the `update-docs` skill instead.

## When to Use

- After landing new game systems, challenges, or mechanics
- When balance constants change (discovery rates, rewards, costs, XP curves)
- New zones, rooms, achievements, or equipment added
- Before a release

## Initialize Wiki Submodule

```bash
# Check status (- prefix = not initialized)
git submodule status quest.wiki

# Initialize
git submodule update --init quest.wiki

# Verify (commit hash without - prefix)
git submodule status quest.wiki
```

If `quest.wiki/` exists as a regular directory, remove it first: `rm -rf quest.wiki && git submodule update --init quest.wiki`

## Wiki Pages

| Page | Content |
|------|---------|
| `Home.md` | Landing page, system overview, quick links, installation |
| `Getting-Started.md` | New player guide, character creation, gameplay loop |
| `Combat.md` | Damage pipeline, enemy scaling, boss multipliers, XP, death |
| `Zones-and-Progression.md` | All zones, subzones, bosses, prestige gates |
| `Prestige.md` | Tiers (Bronze→Eternal), multipliers, combat bonuses |
| `Equipment.md` | Slots, rarities, affixes, drop rates, auto-equip |
| `Dungeons.md` | Discovery, sizes, room types, keys, safe death |
| `Fishing.md` | 40 ranks, 8 tiers, Storm Leviathan hunt |
| `Haven.md` | 14-room skill tree, bonuses, build order |
| `Soulforge.md` | Enhancement +0 to +10, success rates, costs |
| `Challenges.md` | All 10 minigames, controls, difficulties, rewards |
| `Achievements.md` | Categories, achievements, score system, titles |
| `Strategy-Guide.md` | Progression guide (early→endgame), pro tips |
| `Stormbreaker-Path.md` | Walkthrough, PR cost breakdown, checklist |
| `Controls-and-UI.md` | Keyboard reference for every screen |
| `The-Deep.md` | Mercenary system, missions, layers, infrastructure |
| `Stormglass.md` | Currency, Storm Sigils, earning/spending |

## Audit and Update

### What to check:
- New game systems not yet documented
- Changed constants affecting player-facing numbers
- New zones, rooms, achievements, equipment
- Cross-links between pages (`[[Page Name]]` format)

### Scope tiers:

**Small** (1-3 pages): Edit directly, commit, push to `quest.wiki/`.

**Large** (new systems, many pages):
- Create a team:
  - **2 sys-architects**: Research codebase, write to `_research_*.md` temp files
  - **2 product managers**: Write/update wiki pages (split pages between them)
  - **1 game designer**: Strategy and gameplay guidance pages
- Dependencies: research (parallel) → writing (parallel, blocked by research) → final review

### Update rules:
- Player-facing tone: friendly, engaging, practical tips alongside mechanics
- Use `[[Page Name]]` for cross-references
- Data tables for numbers-heavy content
- Translate code concepts to player language ("prestige" not "reset")
- Every page: "See Also" or "Related Pages" section
- Clean up `_research_*.md` temp files before pushing
- Commit directly to `quest.wiki/` (no branch protection)
- Push to `origin master` (wiki uses `master`, not `main`)

## Update Submodule Pointer

After pushing wiki changes, update the main repo's submodule pointer. Include in the same docs PR branch:

```bash
git add quest.wiki
git commit -m "docs: update quest.wiki submodule pointer"
```

Do this BEFORE creating the PR so the pointer update is included.
