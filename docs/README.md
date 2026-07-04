# docs/

**The specification of what Quest does now lives in
[`openspec/specs/`](../openspec/README.md)** — a reverse-engineered,
code-grounded baseline of 20 capabilities (197 requirements) and the single
source of truth. Start non-trivial work from a spec via `/opsx:propose`.

The former specification and design/plan documents that used to live under
`docs/` have been **removed** — they are superseded by `openspec/specs/`:

- Top-level "as implemented" system docs (`system-design.md`, `core-systems.md`,
  `secondary-systems.md`, `challenge-minigames.md`, `infrastructure.md`,
  `balancing.md`).
- `docs/design/`, `docs/plans/`, `docs/archive/` — architecture, UX, and dated
  per-feature design + implementation plans.
- `docs/superpowers/` — feature design specs and plans.

Their full history remains in git; source files that once cited a design doc now
point at the relevant `openspec/specs/<capability>/spec.md`.

## What's still here

`docs/` now holds only design-process **rationale** and **assets** — not the
specification:

| Path | What it is |
|------|-----------|
| `decisions.md` | Design decisions log with rationale (maintained by the `design-iteration` skill) |
| `dossiers/` | Living design dossiers (maintained by the `design-iteration` skill) |
| `reviews/` | Editorial and testing-strategy reviews |
| `storyboards/` | HTML storyboards |
| `screenshots/` | PNG captures used in PR / verification write-ups |

When a design note and the code disagree, trust the code and reconcile the
relevant `openspec/specs/` requirement.
