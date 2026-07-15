# Quest model-eval suite

Measures how well a model works on Quest's codebase using scoped,
deterministic tasks: the model gets a symptom description + selected source
files + failing check output, and must return one unified diff. The harness
applies the diff in an isolated worktree and grades it with the repo's own
graders (tests, snapshot tests, seeded simulators).

Design rationale: [docs/explorations/2026-07-15-model-eval-suite.md](../docs/explorations/2026-07-15-model-eval-suite.md).

## Quick start

```bash
# Integrity-check every fast task (no API key needed):
python3 evals/harness/run.py validate

# Dry-run the full pipeline using the reference solutions as "the model":
python3 evals/harness/run.py run --reference

# Evaluate a real model (needs ANTHROPIC_API_KEY + `pip install anthropic`):
python3 evals/harness/run.py run --model claude-sonnet-5

# Include the slow tier (simulator-graded balance tasks):
python3 evals/harness/run.py run --tier all --model claude-sonnet-5

# Judge pass over a finished run (quality scores on top of pass/fail):
python3 evals/harness/judge.py evals/results/<run-dir>
```

Reports land in `evals/results/<timestamp>-<label>/report.{json,md}`
(gitignored). Per-task artifacts (prompt, raw response, extracted patch,
grader logs) sit alongside for debugging.

## How a task works

```
evals/tasks/<domain>/<task-id>/
├── task.toml            # metadata: graders, context files, edit allowlist
├── prompt.md            # the symptom description the model sees
├── bug.patch            # applied to the eval base commit -> the task state
├── reference.patch      # known-good fix (never shown to the model)
└── failing_output.txt   # grader output on the bug state (shown if enabled)
```

Every task is pinned to one shared **eval base commit**
(`config.toml [suite] base_commit`), so a single warm build serves the whole
fast tier. The runner reuses one scratch worktree (`.worktrees/eval-scratch`)
and the main `target/` dir; between tasks it hard-resets to base.

**Outcome funnel** (each task lands in exactly one):
`apply_error` → `illegal_edit` → `build_error` → `test_fail` → `pass`.
The distinction matters: a model that can't emit applicable diffs needs
different work than one whose fixes are plausible but wrong.

**Anti-gaming**: diffs may only touch the task's `edit_allowlist`, and never
the global denylist (`tests/`, `*.snap`, `evals/`, build files — see
`config.toml`). A model can never re-bless a snapshot, edit a test, or
patch the harness.

**Repair round**: if a diff fails to *apply* (malformed patch), the model
gets exactly one retry seeing only the apply error — never grader feedback.
`--strict` disables even that.

## Validation is the contract

`run.py validate` proves every task is honest:

1. `bug.patch` applies cleanly to the base commit,
2. at least one grader is **red** on the bug state,
3. `reference.patch` applies on top and turns everything **green**,
4. the reference stays inside the edit allowlist.

Run it after authoring or mining tasks, after re-cutting `base_commit`, and
in any PR that touches `evals/tasks/`. `--refresh-failing-output` rewrites
each task's cached `failing_output.txt` from the live red run.

## Adding tasks

**Mined (preferred):**

```bash
git fetch --unshallow                      # once; the CI clone is shallow
python3 evals/harness/mine.py candidates --limit 40
python3 evals/harness/mine.py extract <sha> --id qb-0xx-short-name
```

Skeletons land in `evals/tasks/_incoming/` with the red leg pre-checked.
Then the human pass: rewrite `prompt.md` to describe the **symptom** (what a
player or CI sees — never the fix), tighten `context_files` and
`edit_allowlist`, move the directory to `evals/tasks/<domain>/`, and run
`run.py validate <task-id>`.

**Hand-authored:** create the same file set by hand; author `bug.patch`
against the base commit and make `reference.patch` its exact reverse (the
fix as a model would write it). Validate the same way.

Task ID prefixes: `qb-` bugfix, `qf-` feature, `qu-` ui, `qbal-` balance.

## Prompt-writing rules

- Describe the observable symptom, the affected flow, and the acceptance
  bar. Name the failing tests if `show_failing_output` is off.
- Never name the culprit line, constant, or the shape of the fix.
- Keep `context_files` to what a competent engineer would actually need —
  spoon-feeding exactly one file with one suspicious line makes the task
  trivial; dumping 20 files makes it a context-length test instead.

## Scoring

- **Primary**: pass@1 per domain and overall (deterministic gates only).
- **Diagnostic**: the outcome funnel distribution.
- **Secondary**: judge scores 0-4 (convention / minimality / solution_match)
  from `judge.py`, reported separately and never blended into pass rate.
- pass@k: repeat `run.py run` k times (runs are cheap once warm) and
  aggregate the per-run `report.json` files; a built-in `--samples` flag is
  deliberately deferred until someone actually needs it.

Comparing two models = two `run.py run` invocations + diffing `report.md`.
The suite version + base commit are stamped into every report; results are
only comparable within the same suite version.

## Tiers

| Tier | Graders | Cost | When |
|---|---|---|---|
| `fast` (default) | unit/integration tests, snapshot tests | seconds/task after one warm build | every iteration |
| `slow` | `simulator --check-progression` (release build) | minutes/task | nightly / pre-release / on demand |
