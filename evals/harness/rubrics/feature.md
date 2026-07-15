# Feature rubric

Score each dimension 0-4 (0 = badly violated, 4 = exemplary).

**convention** — The implementation matches Quest's established patterns for
the touched module: input routed through the existing `handle_game_input`
dispatch shape, `InputResult` variants chosen correctly (a save-worthy action
returns `NeedsSaveWithEvent`), module layout (types/logic split) respected,
difficulty-tier and forfeit patterns followed where relevant.

**minimality** — Implements exactly the described behavior. Penalize
speculative extensions, config knobs nobody asked for, and copy-paste that
duplicates an existing helper instead of reusing it.

**solution_match** — Semantic proximity to the reference implementation.
Equally idiomatic alternates score 4; implementations that satisfy the tests
by special-casing the tested inputs score 0.
