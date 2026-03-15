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
