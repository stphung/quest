# Exploring: an eval suite for measuring model performance on Quest

**Status**: pre-commitment design, crystallized through an interactive Q&A
session (decisions logged below). Nothing here is implemented — graduate it
into an `/opsx:propose` change (or just build `evals/` directly, since this
is dev tooling, not game behavior) when someone is ready to build.

**Prompted by**: "I would like to create an eval suite for measuring
performance of a model working on various aspects of quest."

## Decisions locked in with the owner

| Axis | Decision |
|---|---|
| Subject | **Model-only, scoped tasks** — not the full agent loop. Fixed context in, single answer out, deterministic grading. |
| Coverage | **Bug fixing**, **feature implementation**, **UI/TUI rendering**, **balance & simulation reasoning** (all four). |
| Task source | **Mined from git history** (real merged PRs, reverted to create tasks with known-good reference solutions). |
| Grading | **Deterministic gates + LLM judge** — tests/snapshots/simulators decide pass/fail; a rubric judge scores quality on top. |
| Task format | **Files in, diff out** — prompt + selected source files + failing output; model returns a unified diff. |
| Runtime budget | **Fast suite ≈ 15 min** for the main tier; simulator-graded tasks in a separate slow tier. |
| Location | **In-repo, `evals/`** directory. |
| Contamination | **Not a concern** — relative comparison between models/prompts is the use case. |

## Why Quest is a good eval substrate

The repo already contains the hard part of any coding eval — trustworthy,
deterministic graders:

- **`cargo test`** with a large unit/integration surface, including the
  save-compat corpus (`tests/fixtures/saves/`) that catches serde breakage.
- **UI snapshot tests** (insta + ratatui `TestBackend`) — committed `.snap`
  files are pixel-exact ground truth for rendering tasks, and
  `snapshot_rendering_is_deterministic` catches wall-clock/RNG leaks.
- **Seeded simulators** (`simulator --check-progression`, `deep_simulator`,
  `voyage_simulator`) — objective graders for balance reasoning, a domain
  most coding evals can't touch at all.
- **~754 merged PRs** of real history to mine (the working clone is shallow
  at 50 commits; mining requires `git fetch --unshallow`).

## Architecture

```
evals/
├── README.md              # how to run, how to add tasks
├── harness/               # Python runner (uv script, anthropic SDK)
│   ├── run.py             # materialize → prompt → apply diff → grade → report
│   ├── judge.py           # LLM judge pass over passing/failing diffs
│   └── mine.py            # task-mining pipeline (see below)
├── tasks/
│   ├── bugfix/qb-001-crit-double-dip/
│   │   ├── task.yaml      # metadata + grader spec (schema below)
│   │   ├── prompt.md      # what the model is told
│   │   ├── bug.patch      # applied to the eval base commit to create the task state
│   │   └── reference.patch# the known-good solution (never shown to the model)
│   ├── feature/ …
│   ├── ui/ …
│   └── balance/ …         # slow tier
└── results/               # gitignored; JSON + markdown reports per run
```

### `task.yaml` schema

```yaml
id: qb-001-crit-double-dip
domain: bugfix            # bugfix | feature | ui | balance
tier: fast                # fast | slow
source_pr: 412            # provenance, for auditing
context_files:            # exactly what the model sees, in order
  - src/combat/damage.rs
  - src/core/constants.rs
show_failing_output: true # include grader output in the prompt
graders:                  # hard gates, all must pass
  - cargo test --test combat_tests crit_
  - cargo test --lib combat::
edit_allowlist:           # diff may only touch these paths
  - src/combat/**
judge_rubric: bugfix      # which rubric judge.py applies
```

### One base commit, one warm build

All fast-tier tasks are pinned to a **single eval snapshot commit** (a tag,
e.g. `eval-base-2026-07`), with each task's `bug.patch` rebased onto it
during mining. This is the key to the 15-minute budget: the harness builds
the snapshot once (warm `target/`), then each task is
apply-patch → incremental rebuild (~10–30 s) → targeted test filter. Tasks
that can't be rebased onto the snapshot are dropped or hand-adapted rather
than letting the suite fragment across base commits.

The snapshot is re-cut deliberately (e.g. quarterly, or after a big system
lands like Act 2) and tasks re-validated against it — the mining pipeline
makes regeneration cheap.

### Run flow per task

1. `git worktree add` at the snapshot tag, apply `bug.patch`.
2. Render the prompt: `prompt.md` + full contents of `context_files` +
   (optionally) the failing grader output, captured once at mining time so
   the run doesn't pay for a red build.
3. One model call. The model must return a single fenced unified diff.
4. Apply with `git apply --3way`. If it fails to apply, **one** repair round
   (the model sees only the apply error, not grader feedback), then fail as
   `apply_error`. A `--strict` flag disables the repair round for pure
   single-shot measurement.
5. Enforce the `edit_allowlist` — a diff touching tests, `.snap` files,
   `evals/`, or anything outside the allowlist is an automatic
   `illegal_edit` failure. This is what keeps graders trustworthy:
   *the model can never re-bless a snapshot, edit a test, or touch the
   harness.*
6. Run the graders. All green → **pass**; record which gate failed otherwise.
7. Judge pass (async, batched): rubric-scored 0–4 regardless of pass/fail.

## The four domains

### 1. Bug fixing (`bugfix/`, fast tier, ~25 tasks)

Mined by reverse-applying real fix commits onto the snapshot. Grader = the
tests that shipped with (or already covered) the fix. Quest-specific
sub-flavors worth deliberately over-sampling because the repo's conventions
make them distinctive:

- combat/damage-pipeline math (ordering of Giant's Might → Haven →
  prestige → ascension → defense → crit)
- serde/save-compat regressions (grader: `save_compat_tests` on the corpus)
- RNG determinism leaks (grader: `snapshot_rendering_is_deterministic`)
- tick-loop timing (grader: `game_tick_tests`)

### 2. Feature implementation (`feature/`, fast tier, ~15 tasks)

Small, scoped features mined from history (a new title, a new fishing rank
tier, a new input keybinding routed through `handle_game_input`, a new
`TickEvent`). Grader = the tests that shipped with the feature. The prompt
states the required behavior precisely (derived from the PR description +
test names); the test *source* is withheld but expected test names are
listed, so the task is implementation, not spec-guessing. Full
add-a-minigame (15 integration points) is out of scope for model-only
single-shot — that belongs in a future agentic tier.

### 3. UI / TUI rendering (`ui/`, fast tier, ~10 tasks)

Perturb render code (layout math, style/color selection, responsive-tier
branching, overlay composition) and grade against the committed `.snap`
ground truth via `cargo test snapshot` filters. The edit allowlist
categorically excludes `src/ui/snapshots/` — an intentional-looking
re-bless is an `illegal_edit`. Determinism rules (freezable `ui/clock.rs`)
give a second grader for free on several tasks.

### 4. Balance & simulation reasoning (`balance/`, **slow tier**, ~10 tasks)

The distinctive domain. Two shapes:

- **Restore**: revert a real balance retune (e.g. part of the era retune in
  PR #731) and ask the model to restore progression, given the simulator's
  failing assertion output. Grader: `simulator --check-progression`
  (~2–5 min).
- **Hit a target**: "Ascension VII is reached too early at P200 speedrun;
  adjust costs so it lands in tick range X–Y without breaking the other
  scenarios." Grader: progression check + a task-specific assertion run.

Slow tier runs on demand / nightly, not per-iteration. Where possible each
balance task also gets a *fast proxy grader* (a unit test asserting the
derived curve, e.g. cumulative ascension cost) so the fast suite still
carries a thin balance signal.

## Mining pipeline (`mine.py`)

1. `git fetch --unshallow`; list merged PRs via the GitHub API.
2. **Filter**: 5–300 changed lines in `src/`; must touch test files or be
   covered by existing tests; exclude the heavy audit-log noise (the
   majority of recent commits are 1-line `meta-audit` history appends —
   subject-line and path filters remove them).
3. **Classify** domain by path (`src/combat|core` → bugfix/balance,
   `src/ui` → ui, etc.) + subject keywords; human curates the final label.
4. **Validate** the task property that makes grading meaningful:
   - graders **red** after `bug.patch` (task state),
   - graders **green** after `reference.patch` (reference solves it),
   - reference diff stays inside the `edit_allowlist`.
   Tasks failing any leg are auto-rejected.
5. Emit the task directory; a human pass writes/edits `prompt.md` (mined
   prompts start from the PR title/description, rewritten to describe the
   *symptom*, not the solution).

## LLM judge

Runs after deterministic grading, never overrides it. Per-domain rubrics
scoring 0–4 on:

- **Convention compliance** — the CLAUDE.md rules a test can't always catch:
  seeded-RNG cleanliness in the tick loop, Haven-bonus-as-parameter (no
  globals), `serde(default)` on new `GameState` fields, no wall-clock reads
  in render code, comment discipline.
- **Minimality** — did the diff change only what the task required?
- **Solution match** — semantic proximity to `reference.patch` (informative,
  not gating; legitimate alternative fixes score full marks if conventions
  hold).

Judge model pinned per suite version so scores are comparable across runs.

## Metrics & reporting

- **Primary**: pass@1 overall and per domain (all hard gates green).
- **Diagnostic funnel** per task: `apply_error` → `illegal_edit` →
  `build_error` → `test_fail` → `pass`. Separating these matters: a model
  that can't emit applicable diffs needs different work than one that
  writes plausible-but-wrong fixes.
- **Secondary**: mean judge score (reported separately, never blended into
  pass rate); pass@k optional via `--samples k`.
- Output: `results/<run-id>/report.{json,md}` — per-task rows plus domain
  aggregates; markdown report designed for pasting into a PR/issue when
  comparing two models or prompts.

## Suite size and runtime (targets)

| Tier | Tasks | Grading cost | Wall clock |
|---|---|---|---|
| Fast | ~50 (25 bugfix, 15 feature, 10 ui) | warm build + ~10–30 s incremental each | ≤ 15 min sequential after one-time snapshot build |
| Slow | ~10 balance | 2–5 min sim each | ~30–50 min, nightly/on-demand |

Model-call latency overlaps with grading (call task N+1 while N grades), so
the model side shouldn't extend wall clock.

## Deliberately out of scope (for now)

- **Agentic tier** — full Claude Code sessions graded on final repo state
  (where add-a-minigame, spec-workflow adherence, and `drive-game` visual
  verification would live). The task schema above is designed so an agentic
  runner could reuse the same task directories later (same patches, same
  graders; just drop the `context_files` spoon-feeding).
- **Doc/wiki tasks** — gradeable only by judge, weakest signal; the audit
  skills already cover this ground operationally.
- **Contamination mitigation** — explicitly waived; if the use case ever
  shifts to absolute cross-lab claims, add post-cutoff-only mining and
  symbol-renaming mutation.

## Open questions

1. **Snapshot cadence** — re-cut `eval-base` per quarter, or freeze it for
   longitudinal comparability and accept growing drift from HEAD? (Leaning:
   freeze per suite version `v1`, `v2`, …; never mutate a shipped version.)
2. **Feature-task test visibility** — names-only (proposed) vs. full test
   source in the prompt. Names-only measures more, but a badly named test
   makes the task unfair; curation has to catch that.
3. **Harness language** — Python + anthropic SDK is the fast path; a Rust
   harness in-repo would dodge a toolchain but slow iteration. (Leaning
   Python under `evals/harness/`, `uv` single-file scripts, repo stays
   pure-Rust outside `evals/`.)

## History

- **2026-07-15** — initial design session. Eight design decisions taken
  interactively (table at top); repo groundwork checked: shallow clone
  (50 commits, `--unshallow` needed for mining), ~754 merged PRs upstream,
  recent history dominated by 1-line audit-log commits (mining filter
  required).
