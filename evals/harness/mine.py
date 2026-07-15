#!/usr/bin/env python3
"""Task-mining pipeline: turn real Quest history into eval tasks.

Usage:
    mine.py candidates [--limit N] [--since REV]
        List commits that look like viable task material, scored and
        classified by domain. Requires full history:
        `git fetch --unshallow` first if the clone is shallow.

    mine.py extract <commit> --id <task-id> [--domain D]
        Emit a task skeleton into evals/tasks/_incoming/<task-id>/:
        bug.patch (reverse of the commit, src/ only), reference.patch
        (forward), guessed graders/context, and a prompt.md stub from the
        commit message. Then validates the red leg (bug applies to the
        eval base and at least one grader fails). Curate by hand, move it
        into evals/tasks/<domain>/, rewrite prompt.md to describe the
        SYMPTOM (never the solution), and run `run.py validate <task-id>`.

Mined tasks are raw material, not finished tasks — every one needs the
human pass before it counts.
"""

from __future__ import annotations

import re
import subprocess
import sys

import lib
from lib import log, sh

# Subjects that are almost never task material.
NOISE_RE = re.compile(
    r"^(Log |meta-audit|doc-audit|wiki-audit|test-audit|dependency-audit|perf-audit"
    r"|docs?:|chore|style:|Merge |Revert )",
    re.I,
)

# Path prefix -> (domain, grader commands) guesses, mirroring the
# "How to Verify Your Change" table in CLAUDE.md.
DOMAIN_MAP = [
    ("src/ui/", "ui", ["cargo test --lib snapshot"]),
    ("src/combat/", "bugfix", ["cargo test --test combat_tests"]),
    ("src/core/tick", "bugfix", ["cargo test --test game_tick_tests"]),
    ("src/core/constants.rs", "balance", ["cargo run --release --bin simulator -- --check-progression"]),
    ("src/items/", "bugfix", ["cargo test --test item_tests"]),
    ("src/input/", "feature", ["cargo test --bin quest input::replay_tests"]),
    ("src/character/", "bugfix", ["cargo test --test save_compat_tests", "cargo test --test character_tests"]),
    ("src/deep/", "bugfix", ["cargo test --test deep_tests"]),
    ("src/loom/", "bugfix", ["cargo test --test loom_tests"]),
    ("src/zones/", "bugfix", ["cargo test --test zone_tests"]),
    ("src/fishing/", "bugfix", ["cargo test --test fishing_tests"]),
    ("src/challenges/", "feature", ["cargo test --lib challenges"]),
]


def is_task_source(path: str) -> bool:
    """Rust source under src/ — excludes docs, snapshots, and test-only code."""
    return (
        path.startswith("src/")
        and path.endswith(".rs")
        and not path.startswith("src/ui/snapshots/")
    )


def numstat(rev: str) -> list[tuple[int, int, str]]:
    out = sh(["git", "show", "--numstat", "--format=", rev], check=True).stdout
    rows = []
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) == 3 and parts[0].isdigit():
            rows.append((int(parts[0]), int(parts[1]), parts[2]))
    return rows


def classify(paths: list[str]) -> tuple[str, list[str]]:
    for prefix, domain, graders in DOMAIN_MAP:
        if any(p.startswith(prefix) for p in paths):
            return domain, graders
    return "bugfix", ["cargo test"]


def cmd_candidates(args: list[str]) -> int:
    limit = int(args[args.index("--limit") + 1]) if "--limit" in args else 40
    since = args[args.index("--since") + 1] if "--since" in args else None

    if sh(["git", "rev-parse", "--is-shallow-repository"]).stdout.strip() == "true":
        log("NOTE: shallow clone — run `git fetch --unshallow` to mine full history\n")

    rev_range = f"{since}..HEAD" if since else "HEAD"
    commits = sh(
        ["git", "log", "--no-merges", "--format=%H|%s", rev_range], check=True
    ).stdout.splitlines()

    found = 0
    for line in commits:
        if found >= limit:
            break
        sha, subject = line.split("|", 1)
        if NOISE_RE.search(subject):
            continue
        rows = numstat(sha)
        src_rows = [r for r in rows if is_task_source(r[2])]
        touched_tests = [r[2] for r in rows if r[2].startswith("tests/") or "test" in r[2]]
        src_lines = sum(a + d for a, d, _ in src_rows)
        if not src_rows or not (5 <= src_lines <= 300):
            continue
        domain, graders = classify([r[2] for r in src_rows])
        covered = "tests✓" if touched_tests else "tests?"
        log(f"{sha[:10]}  {domain:8} {src_lines:4}L {covered}  {subject[:80]}")
        found += 1
    log(f"\n{found} candidate(s). Extract one with: mine.py extract <sha> --id <task-id>")
    return 0


def cmd_extract(args: list[str]) -> int:
    sha = args[0]
    if "--id" not in args:
        sys.exit("--id <task-id> is required")
    task_id = args[args.index("--id") + 1]

    rows = numstat(sha)
    src_paths = [p for _, _, p in rows if is_task_source(p)]
    if not src_paths:
        sys.exit(f"{sha}: no src/ changes to build a task from")
    domain, graders = classify(src_paths)
    if "--domain" in args:
        domain = args[args.index("--domain") + 1]

    forward = sh(["git", "diff", f"{sha}^", sha, "--", *src_paths], check=True).stdout
    reverse = sh(["git", "diff", sha, f"{sha}^", "--", *src_paths], check=True).stdout
    subject_body = sh(["git", "log", "-1", "--format=%s%n%n%b", sha], check=True).stdout

    tdir = lib.TASKS_DIR / "_incoming" / task_id
    tdir.mkdir(parents=True, exist_ok=True)
    (tdir / "bug.patch").write_text(reverse)
    (tdir / "reference.patch").write_text(forward)
    (tdir / "prompt.md").write_text(
        "<!-- CURATE ME: describe the SYMPTOM a player/CI sees, never the fix.\n"
        f"     Mined from {sha[:12]}. Original message:\n{subject_body.strip()}\n-->\n"
    )
    dirs = sorted({p.rsplit("/", 1)[0] + "/" for p in src_paths})
    (tdir / "task.toml").write_text(
        f'id = "{task_id}"\n'
        f'domain = "{domain}"\n'
        f'tier = "{"slow" if domain == "balance" else "fast"}"\n'
        f'source = "mined:{sha[:12]}"\n'
        f"context_files = {src_paths!r}\n".replace("'", '"')
        + f"graders = {graders!r}\n".replace("'", '"')
        + f"edit_allowlist = {dirs!r}\n".replace("'", '"')
        + f'judge_rubric = "{domain}"\n'
    )

    # Red-leg check: does the bug patch apply to the eval base, and does
    # at least one guessed grader actually fail there?
    wt = lib.ensure_worktree()
    ok, err = lib.apply_patch(wt, tdir / "bug.patch")
    if not ok:
        log(f"WARNING: bug.patch does not apply to base {lib.base_commit()[:12]} — "
            f"needs manual rebase:\n{err[:800]}")
        return 1
    task = lib.Task.load(tdir)
    red = lib.run_graders(task, wt)
    red_fail = [g for g in red if not g.ok]
    if red_fail:
        (tdir / "failing_output.txt").write_text(lib.failing_excerpt(red))
        log(f"red leg OK: {len(red_fail)}/{len(red)} graders fail on the bug state")
    else:
        log("WARNING: no guessed grader fails on the bug state — find the real "
            "covering test or discard this candidate")
    log(f"\nskeleton written to {tdir}\nNext: curate prompt.md + task.toml, move to "
        f"evals/tasks/{domain}/{task_id}/, then run.py validate {task_id}")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if not args or args[0] in {"-h", "--help"}:
        log(__doc__)
        return 0
    if args[0] == "candidates":
        return cmd_candidates(args[1:])
    if args[0] == "extract":
        return cmd_extract(args[1:])
    sys.exit(f"unknown command: {args[0]}")


if __name__ == "__main__":
    sys.exit(main())
