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

> **Known pitfall — `Cargo.lock` is gitignored in this repo.** It has never been
> committed (confirmed via `.gitignore` and empty `git log --all -- Cargo.lock`), so
> any finding that cites a "locked version" or a resolved dependency-graph edge
> (e.g. "Cargo.lock:generic-array") reflects the live, on-disk lockfile at run time
> only — it is not a citable historical artifact and can never be re-derived from git
> history by a later audit or by `meta-audit`'s `git show <sha>:<path>`
> re-verification. When logging such a finding, say so explicitly in `correct_value`
> (e.g. "observed via live `cargo update --dry-run`/`cargo audit` run; Cargo.lock is
> gitignored, not re-derivable from git history") rather than stating a locked
> version as if it were a stable, checkable fact.
>
> **Known pitfall — phantom transitive-dependency findings.** A `meta-audit`
> re-verification pass (2026-07-17) checked the recurring "`generic-array` pinned by
> `crypto-common`'s exact `=0.14.7` requirement via ratatui's optional termwiz
> backend" and "a second `rand` 0.8.x pulled in transitively via criterion's
> `phf_generator`" findings, logged across roughly ten runs since March 2026 with
> inconsistent details each time (0.14.7 sometimes reachable, sometimes "0 packages
> to lock"; the second `rand` cited as both 0.8.6 and 0.8.7). Independent
> verification found **neither dependency edge exists at all** in the current
> resolved graph — `cargo tree -i generic-array` / `cargo tree -i rand` show nothing
> beyond this project's own direct deps, and `ratatui`'s locked deps never include a
> termwiz backend. Before reporting an "orphaned" or "transitive-only" dependency as
> a finding, confirm it's actually reachable with `cargo tree -i <crate>` (or `cargo
> tree -e normal,build,dev | grep -B5 "<crate> v"`) in *this run's* freshly generated
> lockfile — if that comes back empty, it isn't a real finding, don't log it (a
> stale mention in `cargo update --dry-run --verbose`'s unchanged-deps list is not by
> itself evidence of a live edge in the graph).

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

> **Known pitfall — unverified negative claims.** A `meta-audit` re-verification pass
> confirmed two past Agent 2 findings that asserted something was *absent* or *not
> gated to tests* without exhaustively checking: one claimed a dependency was used
> "pervasively in production" outside `#[cfg(test)]` when every cited call site was
> actually inside a test module, and another claimed "no test explicitly asserts X"
> when such a test did exist. Before stating a negative ("not used", "no test covers
> this", "not gated to tests"), grep for the specific counter-evidence (test module
> boundaries, the assertion pattern itself) rather than inferring absence from a
> plausible-sounding hit count.

> **Known pitfall — unverified "is exercised" claims.** A `meta-audit`
> re-verification pass (2026-07-17) found a past run's fix reasoning wrong in the
> other direction from the pitfall above: it justified keeping ratatui's
> `underline-color` feature enabled by claiming it "IS exercised," with no call-site
> evidence cited. Independent re-verification found zero `.underline_color(...)`
> call sites and that every `underline:` field across all committed snapshots was
> `Reset` — the feature was actually dead, and a later run had to catch and remove it
> a full audit cycle after this one kept it. Before asserting a feature or dependency
> capability "is used"/"is exercised" as grounds for keeping it, cite the specific
> call site (or specific non-default rendered value, e.g. a non-`Reset` snapshot
> field) — the same rigor the negative-claims pitfall above demands, applied to
> positive claims too.

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
- **`correct_value` for `auto_fixed: true` findings**: describe the state *found* at
  `commit_sha` (the problem — e.g. "uuid pinned at 1.21, 1.22 available"), not the fix
  that Phase 2 goes on to apply. A past run logged `correct_value` as "Moved tar/flate2
  into `[target...]`.dependencies]" — but that move happens in a commit *after*
  `commit_sha` (inside the same PR), so `git show <commit_sha>:Cargo.toml` will never
  show it and a later `meta-audit` re-verification will wrongly flag the finding as
  false. If you want to record what the fix ended up being, put it in a separate note,
  not in `correct_value`.
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
