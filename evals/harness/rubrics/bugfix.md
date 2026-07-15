# Bugfix rubric

Score each dimension 0-4 (0 = badly violated, 4 = exemplary).

**convention** — Quest/CLAUDE.md conventions a test can't always catch:
seeded-RNG cleanliness in tick-path code (no wall-clock or thread RNG),
Haven bonuses passed as parameters (never globals), `#[serde(default)]` /
`alias` / migration for save-format changes (never editing the corpus),
comment discipline (comments state constraints, not narration of the change).

**minimality** — The diff changes only what the defect requires. Penalize
drive-by refactors, renamed variables, reformatted untouched lines, added
comments explaining the fix to a reviewer.

**solution_match** — Semantic proximity to the reference fix. A different
but equally sound fix that respects the pipeline's documented ordering
scores 4; a fix that happens to pass the graders while distorting adjacent
behavior (e.g. compensating constants) scores 0-1.
