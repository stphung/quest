---
name: audit
description: Run all audit skills — perf-audit, test-audit, doc-audit, wiki-audit, dependency-audit — in parallel on isolated worktrees. Use when you want a full project health check or before a release.
---

# Full Audit

Run all 5 audit skills in parallel on isolated git worktrees for a comprehensive project health check.

## When to Use

- Before a release
- After landing a large feature
- Periodic health check
- When asked to "audit everything" or "full audit"

## Process

Launch all 5 audit skills simultaneously, each on its own worktree. Each skill creates its own branch, PR, and merges independently via `/ship`.

### Parallel Launch

Spawn 5 agents in a **single message** (so they run concurrently), each with `isolation: "worktree"` and `mode: "bypassPermissions"`. Each agent prompt should:

1. Read its corresponding skill file from the repo (e.g., `.claude/skills/perf-audit/SKILL.md`)
2. Follow the skill instructions exactly
3. Create a branch, commit fixes, push, create a PR with automerge, and wait for merge
4. Return a summary of findings and the PR URL

```
Agent 1 — perf-audit   → Read .claude/skills/perf-audit/SKILL.md, execute it fully
Agent 2 — test-audit   → Read .claude/skills/test-audit/SKILL.md, execute it fully
Agent 3 — doc-audit    → Read .claude/skills/doc-audit/SKILL.md, execute it fully
Agent 4 — wiki-audit   → Read .claude/skills/wiki-audit/SKILL.md, execute it fully
Agent 5 — dependency-audit → Read .claude/skills/dependency-audit/SKILL.md, execute it fully
```

**Important**: Since agents run on isolated worktrees, they won't conflict with each other even if they touch overlapping files. Each PR merges independently.

### Agent Prompt Template

Each agent should receive a prompt like:

> You are running a full audit of the Quest codebase. Read the skill file at `.claude/skills/{name}/SKILL.md` and follow its instructions completely. You are working in an isolated git worktree.
>
> Key requirements:
> - Follow every phase described in the skill file
> - Create a branch named `{name}-YYYYMMDD` (use today's date)
> - Commit all changes with a descriptive message
> - Push the branch and create a PR with automerge enabled (`gh pr merge --auto --squash`)
> - Wait for CI to pass and the PR to merge
> - If CI fails, fix the issues and push again
> - Return: (1) summary of findings, (2) fixes applied, (3) PR URL and merge status

## Output

After all 5 agents complete, compile and report a summary table:

| Audit | Findings | Fixes | PR | Status |
|-------|----------|-------|----|--------|
| perf-audit | ... | ... | #N | merged/pending |
| test-audit | ... | ... | #N | merged/pending |
| doc-audit | ... | ... | #N | merged/pending |
| wiki-audit | ... | ... | #N | merged/pending |
| dependency-audit | ... | ... | #N | merged/pending |
