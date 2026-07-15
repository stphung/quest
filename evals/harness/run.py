#!/usr/bin/env python3
"""Quest model-eval runner.

Usage:
    run.py list                          # show all tasks
    run.py validate [ids...] [options]   # integrity-check tasks (no model)
    run.py run [ids...] [options]        # evaluate a model on tasks

Options:
    --tier fast|slow|all      task tier to include (default: fast)
    --model MODEL             model under evaluation (default: config.toml)
    --reference               answer with reference.patch instead of a model
                              (dry-runs the whole pipeline, no API key needed)
    --strict                  no repair round on malformed/unappliable diffs
    --refresh-failing-output  (validate) rewrite each task's failing_output.txt

`validate` proves each task is honest: graders RED on the bug state, GREEN
after reference.patch, and the reference stays inside the edit allowlist.
Run it after authoring or mining any task, and after moving base_commit.
"""

from __future__ import annotations

import json
import sys

import lib
from lib import log


def build_prompt(task: lib.Task, wt) -> str:
    parts = [
        "You are fixing an issue in Quest, a terminal-based idle RPG written in Rust.",
        "Work only from the information below; you cannot run commands or open other files.",
        "",
        "## Task",
        "",
        task.prompt_text.strip(),
        "",
        "## Files",
    ]
    for rel in task.context_files:
        content = (wt / rel).read_text()
        parts += ["", f"### {rel}", "", "```rust", content.rstrip("\n"), "```"]
    if task.show_failing_output and task.failing_output:
        parts += [
            "",
            "## Failing check output",
            "",
            "```",
            task.failing_output.strip(),
            "```",
        ]
    allow = ", ".join(f"`{a}`" for a in task.edit_allowlist)
    parts += [
        "",
        "## Response format (required)",
        "",
        "Reply with exactly ONE unified diff in a fenced ```diff block, using",
        "`--- a/<path>` / `+++ b/<path>` headers with paths relative to the repo root.",
        f"You may only modify files under: {allow}.",
        "Do not modify tests, snapshots, or build files. Keep the change minimal.",
        "Any text outside the fenced diff block is ignored.",
    ]
    return "\n".join(parts)


def grade_diff(task: lib.Task, wt, diff_text: str) -> dict:
    """Apply a candidate diff to the bugged worktree and run the graders."""
    paths = lib.patch_paths(diff_text)
    if not paths:
        return {"outcome": "apply_error", "failed_gate": "no file paths found in diff"}
    violations = lib.path_violations(paths, task.edit_allowlist)
    if violations:
        return {
            "outcome": "illegal_edit",
            "failed_gate": f"touches disallowed paths: {', '.join(violations)}",
        }
    ok, err = lib.apply_patch(wt, diff_text)
    if not ok:
        return {"outcome": "apply_error", "failed_gate": "git apply failed", "apply_error": err}
    graders = lib.run_graders(task, wt)
    outcome = lib.classify(graders)
    failed = next((g.cmd for g in graders if not g.ok), "")
    return {
        "outcome": outcome,
        "failed_gate": failed,
        "grader_log": "\n\n".join(f"$ {g.cmd}\n{g.output[-4000:]}" for g in graders),
    }


def cmd_run(tasks: list[lib.Task], opts: dict) -> int:
    model = opts.get("model") or lib.CONFIG["run"]["model"]
    reference_mode = opts.get("reference", False)
    label = "reference" if reference_mode else model.replace("/", "-")
    run_dir = lib.new_run_dir(label)
    rows = []

    for task in tasks:
        log(f"\n=== {task.id} ({task.domain}/{task.tier}) ===")
        wt = lib.ensure_worktree()
        ok, err = lib.apply_patch(wt, task.bug_patch)
        if not ok:
            sys.exit(f"{task.id}: bug.patch no longer applies to base — re-validate the suite:\n{err}")

        task_dir = run_dir / task.id
        task_dir.mkdir(parents=True)
        prompt = build_prompt(task, wt)
        (task_dir / "prompt.md").write_text(prompt)

        if reference_mode:
            diff_text = task.reference_patch.read_text()
            response = "(reference mode: reference.patch used as the response)"
        else:
            response = lib.call_model(prompt, model, lib.CONFIG["run"]["max_tokens"])
            diff_text = lib.extract_diff(response)
        (task_dir / "response.txt").write_text(response)

        result: dict
        if diff_text is None:
            result = {"outcome": "apply_error", "failed_gate": "no diff block in response"}
        else:
            result = grade_diff(task, wt, diff_text)
        # One repair round: only for responses that never applied (malformed
        # or missing diff), never for grader feedback — that would turn
        # single-shot into iteration.
        if (
            result["outcome"] == "apply_error"
            and not opts.get("strict")
            and not reference_mode
        ):
            repair_prompt = (
                prompt
                + "\n\n## Your previous response could not be applied\n\n```\n"
                + result.get("apply_error", result["failed_gate"])[-2000:]
                + "\n```\n\nResend the complete corrected diff in one ```diff block."
            )
            response = lib.call_model(repair_prompt, model, lib.CONFIG["run"]["max_tokens"])
            (task_dir / "response_repair.txt").write_text(response)
            diff_text = lib.extract_diff(response)
            if diff_text is not None:
                lib.reset_worktree()
                lib.apply_patch(wt, task.bug_patch)
                result = grade_diff(task, wt, diff_text)
                result["repaired"] = True

        if diff_text is not None:
            (task_dir / "model.patch").write_text(diff_text)
        if "grader_log" in result:
            (task_dir / "graders.txt").write_text(result.pop("grader_log"))

        row = {"task": task.id, "domain": task.domain, "tier": task.tier, **result}
        (task_dir / "result.json").write_text(json.dumps(row, indent=2))
        rows.append(row)
        log(f"    -> {row['outcome']}" + (f" ({row['failed_gate']})" if row["outcome"] != "pass" else ""))

    lib.write_report(
        run_dir,
        rows,
        {"mode": "reference" if reference_mode else "model", "model": None if reference_mode else model},
    )
    return 0 if all(r["outcome"] == "pass" for r in rows) else 1


def cmd_validate(tasks: list[lib.Task], opts: dict) -> int:
    failures = []
    for task in tasks:
        log(f"\n=== validate {task.id} ({task.domain}/{task.tier}) ===")

        ref_paths = lib.patch_paths(task.reference_patch.read_text())
        violations = lib.path_violations(ref_paths, task.edit_allowlist)
        if violations:
            failures.append(f"{task.id}: reference.patch outside allowlist: {violations}")
            log("    FAIL: reference outside allowlist")
            continue

        wt = lib.ensure_worktree()
        ok, err = lib.apply_patch(wt, task.bug_patch)
        if not ok:
            failures.append(f"{task.id}: bug.patch does not apply: {err[:500]}")
            log("    FAIL: bug.patch does not apply")
            continue

        red = lib.run_graders(task, wt)
        red_fail = [g for g in red if not g.ok]
        if not red_fail:
            failures.append(f"{task.id}: graders all GREEN on bug state — task detects nothing")
            log("    FAIL: no grader is red on the bug state")
            continue
        log(f"    red OK: {len(red_fail)}/{len(red)} graders fail on bug state")
        if opts.get("refresh_failing_output"):
            (task.dir / "failing_output.txt").write_text(lib.failing_excerpt(red))
            log("    wrote failing_output.txt")

        ok, err = lib.apply_patch(wt, task.reference_patch)
        if not ok:
            failures.append(f"{task.id}: reference.patch does not apply on bug state: {err[:500]}")
            log("    FAIL: reference.patch does not apply")
            continue

        green = lib.run_graders(task, wt)
        green_fail = [g for g in green if not g.ok]
        if green_fail:
            failures.append(
                f"{task.id}: graders still RED after reference: {[g.cmd for g in green_fail]}"
            )
            log(f"    FAIL: still red after reference: {[g.cmd for g in green_fail]}")
            continue
        log("    green OK: all graders pass after reference.patch")

    log("\n" + "=" * 60)
    if failures:
        log(f"VALIDATE FAILED ({len(failures)}):")
        for f in failures:
            log(f"  - {f}")
        return 1
    log(f"VALIDATE OK: {len(tasks)} task(s) honest (red on bug, green on reference)")
    return 0


def cmd_list(tasks: list[lib.Task], _opts: dict) -> int:
    for t in tasks:
        log(f"{t.id:28} {t.domain:8} {t.tier:5} graders={len(t.graders)}  {t.dir.relative_to(lib.EVALS_DIR)}")
    log(f"\n{len(tasks)} task(s)")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if not args or args[0] in {"-h", "--help"}:
        log(__doc__)
        return 0
    cmd, rest = args[0], args[1:]

    opts: dict = {"tier": "fast"}
    ids = []
    i = 0
    while i < len(rest):
        a = rest[i]
        if a == "--tier":
            opts["tier"] = rest[i + 1]; i += 2
        elif a == "--model":
            opts["model"] = rest[i + 1]; i += 2
        elif a == "--reference":
            opts["reference"] = True; i += 1
        elif a == "--strict":
            opts["strict"] = True; i += 1
        elif a == "--refresh-failing-output":
            opts["refresh_failing_output"] = True; i += 1
        else:
            ids.append(a); i += 1

    tier = "all" if cmd == "list" and "--tier" not in sys.argv else opts["tier"]
    tasks = lib.discover_tasks(ids or None, tier=tier)
    if not tasks:
        sys.exit("no tasks matched")

    if cmd == "list":
        return cmd_list(tasks, opts)
    if cmd == "validate":
        return cmd_validate(tasks, opts)
    if cmd == "run":
        return cmd_run(tasks, opts)
    sys.exit(f"unknown command: {cmd}")


if __name__ == "__main__":
    sys.exit(main())
