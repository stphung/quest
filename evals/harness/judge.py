#!/usr/bin/env python3
"""LLM-judge pass over a completed eval run.

Usage:
    judge.py <results-run-dir> [--model MODEL]

For every task in the run that produced a diff, scores it 0-4 on the
domain rubric (evals/harness/rubrics/<rubric>.md). Judge scores are
QUALITY signal only — they never override the deterministic outcome.
Writes judge.json next to each task's result.json and a judge summary
into the run's report.json.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

import lib
from lib import log

RUBRICS_DIR = Path(__file__).resolve().parent / "rubrics"

JUDGE_JSON_RE = re.compile(r"\{.*\}", re.S)


def judge_prompt(task: lib.Task, model_patch: str, outcome: str) -> str:
    rubric = (RUBRICS_DIR / f"{task.judge_rubric}.md").read_text()
    return "\n".join(
        [
            "You are grading a model-produced patch for a Rust game codebase (Quest).",
            "Score strictly per the rubric. The deterministic grader outcome is given;",
            "do not re-litigate correctness — judge quality dimensions only.",
            "",
            "## Rubric",
            rubric.strip(),
            "",
            "## Task given to the model",
            task.prompt_text.strip(),
            "",
            f"## Deterministic outcome: {outcome}",
            "",
            "## Model patch",
            "```diff",
            model_patch.strip(),
            "```",
            "",
            "## Reference patch (a known-good solution; alternatives may be equally valid)",
            "```diff",
            task.reference_patch.read_text().strip(),
            "```",
            "",
            "Respond with ONLY a JSON object:",
            '{"convention": 0-4, "minimality": 0-4, "solution_match": 0-4, "rationale": "<2 sentences>"}',
        ]
    )


def main() -> int:
    args = sys.argv[1:]
    if not args:
        log(__doc__)
        return 0
    run_dir = Path(args[0])
    model = lib.CONFIG["judge"]["model"]
    if "--model" in args:
        model = args[args.index("--model") + 1]

    report_path = run_dir / "report.json"
    report = json.loads(report_path.read_text())
    tasks_by_id = {t.id: t for t in lib.discover_tasks(tier="all")}

    for row in report["tasks"]:
        task = tasks_by_id.get(row["task"])
        patch_file = run_dir / row["task"] / "model.patch"
        if task is None or not patch_file.exists():
            continue
        prompt = judge_prompt(task, patch_file.read_text(), row["outcome"])
        response = lib.call_model(prompt, model, lib.CONFIG["judge"]["max_tokens"])
        m = JUDGE_JSON_RE.search(response)
        scores = json.loads(m.group(0)) if m else {"error": "unparseable judge response"}
        (run_dir / row["task"] / "judge.json").write_text(json.dumps(scores, indent=2))
        row["judge"] = scores
        log(f"{row['task']}: {scores}")

    report["meta"]["judge_model"] = model
    report_path.write_text(json.dumps(report, indent=2))
    log(f"\njudge scores merged into {report_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
