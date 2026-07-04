# docs/

**The specification of what Quest currently does now lives in
[`openspec/specs/`](../openspec/README.md)** — a reverse-engineered,
code-grounded baseline of 20 capabilities (197 requirements). That is the single
source of truth. Start non-trivial work from a spec via `/opsx:propose`.

The top-level "as implemented" system documents that used to live here
(`system-design.md`, `core-systems.md`, `secondary-systems.md`,
`challenge-minigames.md`, `infrastructure.md`, `balancing.md`) have been
**removed** — they duplicated that specification role and are superseded by
`openspec/specs/`. Their history remains in git.

## What's still here

`docs/` now holds **design-process history and assets**, not the specification:

| Path | What it is |
|------|-----------|
| `decisions.md` | Design decisions log with rationale (maintained by the `design-iteration` skill) |
| `dossiers/` | Living design dossiers (maintained by the `design-iteration` skill) |
| `design/` | Architecture / UX / balance design write-ups (the *how* and *why*) |
| `plans/` | Dated per-feature design + implementation plans (historical) |
| `archive/` | Older archived plans |
| `reviews/` | Editorial and testing-strategy reviews |
| `superpowers/` | Feature design specs & plans (source `//!` comments cite these for design provenance) |
| `storyboards/` | HTML storyboards |
| `screenshots/` | PNG captures used in PR/verification write-ups |

These describe how and why the game was built. OpenSpec's behavioral
requirements describe what it does; the two are complementary. When a design doc
and the code disagree, trust the code and reconcile the relevant
`openspec/specs/` requirement.
