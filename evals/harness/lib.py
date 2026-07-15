"""Shared plumbing for the Quest model-eval harness.

Zero third-party dependencies: stdlib only (tomllib needs Python 3.11+).
The optional `anthropic` package is imported lazily and only when a real
model call is requested (run.py without --reference, or judge.py).
"""

from __future__ import annotations

import datetime
import json
import os
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

EVALS_DIR = Path(__file__).resolve().parent.parent
REPO = EVALS_DIR.parent
TASKS_DIR = EVALS_DIR / "tasks"
RESULTS_DIR = EVALS_DIR / "results"
SCRATCH_WT = REPO / ".worktrees" / "eval-scratch"

CONFIG = tomllib.loads((EVALS_DIR / "config.toml").read_text())

# Funnel outcomes, ordered from "never even applied" to "solved".
OUTCOMES = ["apply_error", "illegal_edit", "build_error", "test_fail", "pass"]

BUILD_ERROR_RE = re.compile(r"error\[E\d+\]|error: could not compile|error: linking")


def log(msg: str) -> None:
    print(msg, flush=True)


def sh(
    cmd: list[str] | str,
    cwd: Path = REPO,
    env: dict | None = None,
    check: bool = True,
    timeout: int = 1800,
) -> subprocess.CompletedProcess:
    """Run a command, capturing combined output as text."""
    shell = isinstance(cmd, str)
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        shell=shell,
        env=full_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(
            f"command failed ({proc.returncode}): {cmd}\n{proc.stdout[-4000:]}"
        )
    return proc


# ── Tasks ────────────────────────────────────────────────────────────────


@dataclass
class Task:
    id: str
    domain: str
    tier: str
    dir: Path
    context_files: list[str]
    graders: list[str]
    edit_allowlist: list[str]
    judge_rubric: str
    show_failing_output: bool = True
    source: str = "hand-authored"

    @property
    def prompt_text(self) -> str:
        return (self.dir / "prompt.md").read_text()

    @property
    def bug_patch(self) -> Path:
        return self.dir / "bug.patch"

    @property
    def reference_patch(self) -> Path:
        return self.dir / "reference.patch"

    @property
    def failing_output(self) -> str | None:
        p = self.dir / "failing_output.txt"
        return p.read_text() if p.exists() else None

    @classmethod
    def load(cls, tdir: Path) -> "Task":
        meta = tomllib.loads((tdir / "task.toml").read_text())
        return cls(
            id=meta["id"],
            domain=meta["domain"],
            tier=meta.get("tier", "fast"),
            dir=tdir,
            context_files=meta["context_files"],
            graders=meta["graders"],
            edit_allowlist=meta["edit_allowlist"],
            judge_rubric=meta.get("judge_rubric", meta["domain"]),
            show_failing_output=meta.get("show_failing_output", True),
            source=meta.get("source", "hand-authored"),
        )


def discover_tasks(ids: list[str] | None = None, tier: str = "fast") -> list[Task]:
    tasks = []
    for toml_path in sorted(TASKS_DIR.glob("*/*/task.toml")):
        if "_incoming" in toml_path.parts:
            continue
        task = Task.load(toml_path.parent)
        if ids and task.id not in ids:
            continue
        if tier != "all" and not ids and task.tier != tier:
            continue
        tasks.append(task)
    if ids:
        missing = set(ids) - {t.id for t in tasks}
        if missing:
            sys.exit(f"unknown task ids: {', '.join(sorted(missing))}")
    return tasks


# ── Worktree management ──────────────────────────────────────────────────


def base_commit() -> str:
    return CONFIG["suite"]["base_commit"]


def ensure_worktree() -> Path:
    """Create (or reset) the shared scratch worktree at the eval base commit.

    One worktree is reused for every task, reset between tasks, so cargo's
    incremental cache in the shared target dir stays warm.
    """
    base = base_commit()
    if not SCRATCH_WT.exists():
        sh(["git", "worktree", "prune"])
        sh(["git", "worktree", "add", "--detach", str(SCRATCH_WT), base])
    reset_worktree()
    return SCRATCH_WT


def reset_worktree() -> None:
    sh(["git", "-C", str(SCRATCH_WT), "reset", "--hard", base_commit()])
    sh(["git", "-C", str(SCRATCH_WT), "clean", "-fdq"])


# ── Patches ──────────────────────────────────────────────────────────────


def apply_patch(wt: Path, patch: str | Path) -> tuple[bool, str]:
    """Apply a unified diff in the worktree. Returns (ok, error_output)."""
    if isinstance(patch, Path):
        patch_text = patch.read_text()
    else:
        patch_text = patch
    if not patch_text.endswith("\n"):
        patch_text += "\n"
    for extra in ([], ["--recount", "-C1"]):
        proc = subprocess.run(
            ["git", "apply", "--whitespace=nowarn", *extra, "-"],
            cwd=wt,
            input=patch_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        if proc.returncode == 0:
            return True, ""
    return False, proc.stdout


def patch_paths(patch_text: str) -> list[str]:
    """Every repo-relative path a diff touches (old and new sides)."""
    paths: set[str] = set()
    for m in re.finditer(r"^diff --git a/(\S+) b/(\S+)", patch_text, re.M):
        paths.update(m.groups())
    for m in re.finditer(r"^(?:---|\+\+\+) [ab]/(\S+)", patch_text, re.M):
        paths.add(m.group(1))
    paths.discard("/dev/null")
    return sorted(paths)


def path_violations(paths: list[str], allowlist: list[str]) -> list[str]:
    """Paths that fall outside the task allowlist or inside the global denylist."""
    suite = CONFIG["suite"]
    bad = []
    for p in paths:
        denied = (
            any(p.startswith(pre) for pre in suite["denied_prefixes"])
            or p in suite["denied_files"]
            or any(p.endswith(suf) for suf in suite["denied_suffixes"])
        )
        allowed = any(p.startswith(a) for a in allowlist)
        if denied or not allowed:
            bad.append(p)
    return bad


# ── Graders ──────────────────────────────────────────────────────────────


@dataclass
class GraderResult:
    cmd: str
    ok: bool
    output: str
    build_error: bool = False


def run_graders(task: Task, wt: Path) -> list[GraderResult]:
    timeout_key = "slow_grader_timeout" if task.tier == "slow" else "fast_grader_timeout"
    timeout = CONFIG["run"][timeout_key]
    env = {
        "CARGO_TARGET_DIR": str(REPO / "target"),
        "INSTA_UPDATE": "no",
    }
    results = []
    for cmd in task.graders:
        try:
            proc = sh(cmd, cwd=wt, env=env, check=False, timeout=timeout)
            out, code = proc.stdout, proc.returncode
        except subprocess.TimeoutExpired as e:
            out, code = (e.output or "") + "\n[grader timed out]", 124
        results.append(
            GraderResult(
                cmd=cmd,
                ok=code == 0,
                output=out,
                build_error=code != 0 and bool(BUILD_ERROR_RE.search(out)),
            )
        )
    return results


def classify(graders: list[GraderResult]) -> str:
    if all(g.ok for g in graders):
        return "pass"
    if any(g.build_error for g in graders):
        return "build_error"
    return "test_fail"


def failing_excerpt(graders: list[GraderResult], limit: int = 6000) -> str:
    failed = [g for g in graders if not g.ok]
    per = limit // max(1, len(failed))
    chunks = []
    for g in failed:
        # Drop passing-test noise so the excerpt is all signal.
        kept = "\n".join(
            line for line in g.output.splitlines() if not line.endswith("... ok")
        )
        chunks.append(f"$ {g.cmd}\n{kept[-per:]}")
    return "\n\n".join(chunks)


# ── Model calls ──────────────────────────────────────────────────────────


def call_model(prompt: str, model: str, max_tokens: int) -> str:
    try:
        import anthropic
    except ImportError:
        sys.exit(
            "the `anthropic` package is required for live model calls "
            "(pip install anthropic), or use --reference for a dry run"
        )
    client = anthropic.Anthropic()
    msg = client.messages.create(
        model=model,
        max_tokens=max_tokens,
        messages=[{"role": "user", "content": prompt}],
    )
    return "".join(b.text for b in msg.content if b.type == "text")


DIFF_BLOCK_RE = re.compile(r"```(?:diff|patch)?\n(.*?)```", re.S)


def extract_diff(response: str) -> str | None:
    """Pull the last plausible unified diff out of a model response."""
    candidates = [
        m.group(1)
        for m in DIFF_BLOCK_RE.finditer(response)
        if re.search(r"^(--- |\+\+\+ |diff --git )", m.group(1), re.M)
    ]
    if candidates:
        return candidates[-1]
    # Unfenced fallback: the whole response looks like a diff.
    if re.match(r"^(diff --git |--- )", response.strip()):
        return response.strip() + "\n"
    return None


# ── Reporting ────────────────────────────────────────────────────────────


def new_run_dir(label: str) -> Path:
    stamp = datetime.datetime.now(datetime.UTC).strftime("%Y%m%d-%H%M%S")
    run_dir = RESULTS_DIR / f"{stamp}-{label}"
    run_dir.mkdir(parents=True, exist_ok=False)
    return run_dir


def write_report(run_dir: Path, rows: list[dict], meta: dict) -> None:
    report = {"meta": meta, "tasks": rows}
    (run_dir / "report.json").write_text(json.dumps(report, indent=2))

    by_domain: dict[str, list[dict]] = {}
    for r in rows:
        by_domain.setdefault(r["domain"], []).append(r)

    lines = [
        f"# Eval run: {run_dir.name}",
        "",
        f"- suite: {CONFIG['suite']['version']} @ `{base_commit()[:12]}`",
        *[f"- {k}: {v}" for k, v in meta.items()],
        "",
        "| task | domain | outcome | failed gate |",
        "|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f"| {r['task']} | {r['domain']} | {r['outcome']} | {r.get('failed_gate', '')} |"
        )
    lines += ["", "## By domain", ""]
    for domain, rs in sorted(by_domain.items()):
        passed = sum(1 for r in rs if r["outcome"] == "pass")
        lines.append(f"- **{domain}**: {passed}/{len(rs)} pass")
    total_pass = sum(1 for r in rows if r["outcome"] == "pass")
    lines += ["", f"**Total: {total_pass}/{len(rows)} pass**", ""]
    (run_dir / "report.md").write_text("\n".join(lines))
    log("\n".join(lines[-(len(by_domain) + 4) :]))
    log(f"\nreport: {run_dir / 'report.md'}")
