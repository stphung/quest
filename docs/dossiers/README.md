# System Dossiers

Living design-understanding documents, one per game system, maintained by the
`design-iteration` skill (`.claude/skills/design-iteration/SKILL.md`).

A dossier is the refreshable answer to "what is this system, how does it
interrelate, how is it balanced, and is it fun" — written player-eye first,
with constants cited to source and balance claims backed by simulator runs.
Each carries a `Last refreshed: <date> @ <sha>` header so refreshes can diff
against what landed since.

Dossiers' factual sections (Mechanics & Constants, Interrelations) are in
`doc-audit`'s scope, so stale facts surface in doc audits and get re-verified
by `meta-audit` transitively. Judgment sections (Balance Evidence, Fun
Assessment) are dated snapshots checked by design-iteration's own decision
retrospectives, not by the audits.

Dossiers complement, not replace:
- `docs/plans/` — point-in-time design intent (what a system was *meant* to be)
- `docs/decisions.md` — the log of resolved design decisions
- `src/<module>/CLAUDE.md` — implementation-eye documentation for developers

To create or refresh one, ask in natural language: "where are we with Act 2",
"dissect the Deep", "is the Loom fun", "design review of X".
