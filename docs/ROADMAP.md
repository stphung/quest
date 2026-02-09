# Quest Roadmap

> **Status**: Work in progress. This roadmap reflects current thinking, not commitments. Ideas may be added, removed, or reshuffled as development continues.

## Overview

Quest is a terminal-based idle RPG with core gameplay complete through Zone 10. The next major milestone is **The Expanse** — a post-endgame expansion that adds strategic depth through interconnected systems.

---

## 🎯 The Expanse Expansion

*The flagship update. All systems below are designed to work together in the post-endgame zone.*

### Vision

The Expanse transforms the endgame from "infinite grinding" into a strategic layer cake:
- Build a **party** of characters with different **jobs**
- Hire **mercenaries** for tough fights
- Raise **pets** that grow alongside you
- Construct **factories** to generate resources
- Explore the **map** to find new opportunities
- Complete **temple trials** to unlock systems
- **Upgrade** your gear with earned gold

### Systems

| Issue | System | Role in Expanse |
|-------|--------|-----------------|
| [#72](https://github.com/stphung/quest/issues/72) | **Mercenary** | Hire temporary allies for Expanse encounters |
| [#48](https://github.com/stphung/quest/issues/48) | **Party** | Manage multiple characters tackling Expanse content |
| [#96](https://github.com/stphung/quest/issues/96) | **Pet** | Permanent companions that evolve through Expanse cycles |
| [#95](https://github.com/stphung/quest/issues/95) | **Job** | Class system — level jobs by running Expanse content |
| [#94](https://github.com/stphung/quest/issues/94) | **Upgrade** | Enhance gear using Expanse rewards |
| [#86](https://github.com/stphung/quest/issues/86) | **Map** | Navigate Expanse regions, discover points of interest |
| [#78](https://github.com/stphung/quest/issues/78) | **Factory** | Build automated harvesters in Expanse territory |
| [#98](https://github.com/stphung/quest/issues/98) | **Temple Trials** | Unlock each system through navigation challenges |

### Build Order

```
┌─────────────────────────────────────────────────────────────────┐
│                    EXPANSE SYSTEM DEPENDENCIES                   │
└─────────────────────────────────────────────────────────────────┘

                        [Temple Trials]
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
           [Map]         [Mercenary]      [Factory]
              │               │               │
              │               ▼               │
              │           [Party]             │
              │            /   \              │
              │           ▼     ▼             │
              │        [Pet]  [Job]           │
              │           \   /               │
              └────────────▶│◀────────────────┘
                           ▼
                       [Upgrade]
                    (uses all outputs)
```

**Recommended implementation order:**

1. **Temple Trials** — Gate system that unlocks everything else
2. **Map** — Navigation foundation for Expanse exploration
3. **Mercenary** — Simplest companion system (hire/expire)
4. **Party** — Multi-character management
5. **Factory** — Resource generation (needs map locations)
6. **Pet** — Permanent companions (extends party)
7. **Job** — Class system (extends party/characters)
8. **Upgrade** — Sink for all resources (needs gold, materials, etc.)

---

## 🔨 Now

*Current focus.*

| Issue | Feature | Notes |
|-------|---------|-------|
| [#99](https://github.com/stphung/quest/issues/99) | Responsive UI | Adapt layout to terminal size — foundation for Expanse UI |

---

## 📋 Next

*After responsive UI, begin Expanse systems.*

| Priority | System | Why |
|----------|--------|-----|
| 1 | Temple Trials | Unlocks all other systems narratively |
| 2 | Map | Needed for navigation, factory placement |
| 3 | Mercenary | First companion system, simplest scope |

---

## 💡 Ideas

*Not part of Expanse, but possible future additions.*

- **Gold System** ([#28](https://github.com/stphung/quest/issues/28)) — May merge into Upgrade system
- **Zones 11-20** ([#20](https://github.com/stphung/quest/issues/20)) — Future expansion beyond Expanse
- **Achievements v2** — More categories, tangible rewards
- **Leaderboards** — Optional online rankings
- **Daily/Weekly Challenges** — Rotating objectives
- **Cosmetics** — Portraits, colors, themes
- **New Minigames** — Additional challenge types
- **Multiplayer** — Shared Haven, trade, co-op (major scope)
- **Modding Support** — Custom content via config

---

## ❌ Not Doing

*Ideas considered and rejected.*

| Idea | Why Not |
|------|---------|
| 20 zones (original design) | Expanse provides infinite endgame without content bloat |
| Binary save format | JSON is debuggable, saves are small |
| Per-zone weapon forging | Single Stormbreaker quest is cleaner |
| Exponential prestige (`1.2^rank`) | Trivializes late game |

---

## Design Principles

1. **Expanse-first** — New systems should enhance post-endgame, not bloat early game
2. **Interconnected** — Systems should create synergies (pet + job combos, factory + upgrade loops)
3. **Idle-friendly** — Features work with AFK play
4. **Terminal-first** — Must work in 80×24
5. **Solo-maintainable** — Scope to what one person can build

---

*Last updated: 2026-02-08*
