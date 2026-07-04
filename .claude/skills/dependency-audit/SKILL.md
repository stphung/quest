---
name: dependency-audit
description: Multi-agent dependency health audit — finds outdated versions, unused deps, security issues, and over-specified features. Use when dependencies are stale, after adding deps, or before releases.
---

# Dependency Audit

Full dependency health check: outdated versions, unused dependencies, security advisories, and feature hygiene.

## When to Use

- After adding or removing dependencies
- Before a release
- Periodic health check
- When asked to audit dependencies

## Phase 1: Parallel Audit (2 Agents, Read-Only)

Spawn 2 Explore agents simultaneously.

**Agent 1 — Versions & Security**

Scope: `Cargo.toml`, `Cargo.lock`

Tasks:
1. Run `cargo update --dry-run` to find available compatible updates
2. Run `cargo update --dry-run --verbose` to also see unchanged deps behind latest
3. Run `cargo audit` for security advisories (not just yanked — full advisory DB)
4. For each direct dependency in `[dependencies]`, check if a major version bump is available by reading `Cargo.toml` and comparing against crates.io (use `cargo search <crate>` for each)
5. Check if any dependency has been deprecated or archived

Produce a ranked report:

| Dep | Current | Available | Type | Severity | Notes |
|-----|---------|-----------|------|----------|-------|
| foo | 1.2 | 1.3 | patch | LOW | Compatible bump |
| bar | 2.0 | 3.0 | major | MEDIUM | Breaking change |
| baz | 1.0 | — | security | HIGH | Advisory RUSTSEC-... |

**Agent 2 — Hygiene**

Scope: `Cargo.toml`, `src/`

Tasks:
1. For each direct dependency in `[dependencies]`, grep the codebase for its usage (`use <crate>`, `<crate>::`, or the crate name with hyphens as underscores)
2. Check for over-specified features: compare enabled features in `Cargo.toml` against actual usage patterns in source code
3. Check for deps that might be replaceable with std library alternatives
4. Check `[dev-dependencies]` usage — are they all used in tests/benches?
5. Check for duplicate functionality (two deps doing the same thing)

Produce a ranked report:

| Pattern | Dep | Severity | Notes |
|---------|-----|----------|-------|
| Potentially unused | foo | HIGH | No `use foo` or `foo::` found in src/ |
| Unused feature | bar/feat | MEDIUM | Feature `feat` enabled but not used |
| Std alternative | baz | LOW | Could use std::fs instead |

## Phase 2: Fix (Sequential)

### Auto-fix (no user approval needed)

- Run `cargo update` to apply compatible patch/minor bumps
- Remove confirmed-unused dependencies from `Cargo.toml`
- Remove confirmed-unused features from dependency entries

### Flag for user review

- Major version bumps (breaking API changes)
- Removing deps where usage is ambiguous (re-exported, used in macros, cfg-gated)
- Replacing a dep with a std alternative (behavioral differences possible)
- Any dep with a security advisory that requires code changes

## Phase 3: Verify

1. `make check` must pass (format, lint, test, build, audit)
2. `cargo update --dry-run` shows no remaining compatible updates
3. Report summary:

| Category | Count | Details |
|----------|-------|---------|
| Patch/minor bumps applied | N | list |
| Unused deps removed | N | list |
| Unused features trimmed | N | list |
| Security advisories | N | list |
| Flagged for review | N | list |

## Output

Report the PR URL and final status when done (use `/ship` skill).

## Log This Run

`commit_sha` and the PR URL must be captured correctly, or `meta-audit`'s later
re-verification will check the wrong code state or lose track of provenance:

- **`commit_sha`**: `git merge-base HEAD origin/main` — the commit `main` was at when
  this audit's agents did their read-only cross-referencing, i.e. the exact state every
  finding's `correct_value` describes. Do NOT use `git rev-parse HEAD` (that's this
  branch's own commit, not the code being audited) or the PR's eventual merge commit
  (for skills that modify the audited code itself — e.g. perf-audit, test-audit — the
  merge commit contains the *fix*, not the pre-fix state a finding describes). Capture
  this before the branch is deleted.
- **PR URL**: from `/ship`'s own reported result, once it completes.

1. Build a JSON summary: date (YYYY-MM-DD), the `commit_sha` above, the PR URL, agent
   count, the scope actually covered, and every finding (location, claim, correct value,
   severity, category, whether auto-fixed). Example:
   ```json
   {
     "type": "run",
     "date": "2026-07-10",
     "commit_sha": "abc1234...",
     "pr_url": "https://github.com/stphung/quest/pull/999",
     "agent_count": 2,
     "scope": ["Cargo.toml"],
     "findings": [
       {
         "location": "Cargo.toml:uuid",
         "claim": "...",
         "correct_value": "...",
         "severity": "LOW",
         "category": "patch-bump",
         "auto_fixed": true
       }
     ]
   }
   ```
   `findings` is `[]` for an all-clear run — still log it, it counts toward the threshold.
2. Write it to a temp file and run:
   ```bash
   scripts/audit-eval-log.sh dependency-audit /tmp/dependency-audit-run.json
   ```
3. Check the threshold:
   ```bash
   scripts/audit-eval-check.sh dependency-audit
   ```
   If it prints `TRIGGER`, invoke the `meta-audit` skill for `dependency-audit` next. If
   it prints `SKIP: n/5`, nothing further to do.
4. Commit the updated history log on a small new branch and land it on `main` via the
   same branch+PR+`/ship` convention used for the audit fix itself — this file lives in
   the main repo and needs its own merge to become visible to future runs.
