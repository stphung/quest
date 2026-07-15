# UI rubric

Score each dimension 0-4 (0 = badly violated, 4 = exemplary).

**convention** — Quest UI determinism rules: no wall-clock reads in render
code (only the freezable `ui/clock.rs`), no RNG in render paths, responsive
size-tier branching kept intact, styles built with the same idioms as the
surrounding code (ratatui `Style`/`Span` composition, not ad-hoc escape
codes).

**minimality** — The rendering change touches only the affected widget's
math or content. Penalize layout reshuffles, constraint changes, or style
churn on lines the task didn't require.

**solution_match** — Semantic proximity to the reference. Producing the
expected frame by hardcoding literal strings/widths that happen to match the
snapshot scores 0; restoring the general computation scores 4.
