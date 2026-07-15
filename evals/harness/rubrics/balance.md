# Balance rubric

Score each dimension 0-4 (0 = badly violated, 4 = exemplary).

**convention** — Balance changes live in the documented constants
(`core/constants.rs` tables and named multipliers), keep the documented
curve shapes (e.g. fracture 1.6x, Loom 1.25x scaling), and stay consistent
with the numbers CLAUDE.md documents. Penalize magic numbers scattered into
logic code instead of the constants table.

**minimality** — Adjusts the smallest set of knobs that restores the
progression target. Penalize shotgun retunes of unrelated zones/systems.

**solution_match** — Semantic proximity to the reference retune. A different
knob that legitimately restores pacing (and would plausibly survive the full
progression gate) can score 3-4; compensating hacks that mask the symptom
(e.g. buffing player XP globally to outrun an enemy-stat inflation) score
0-1 even when the simulator passes.
