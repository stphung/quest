# PR Checks System Design

**Date:** 2026-02-01
**Status:** Approved
**Goal:** Comprehensive PR check system to ensure code quality before merging

## Overview

Add strict PR checks to catch issues early while maintaining developer velocity. All checks run automatically on every PR and can be run locally before pushing.

## Design Decisions

### Philosophy
- **Strict quality gates:** Block PRs on format, lint, and test failures
- **Security awareness:** Monitor vulnerabilities but don't block on low-risk warnings
- **Fast feedback:** Quick checks run before expensive multi-platform builds
- **Local-first:** All checks runnable locally with same results as CI

### Checks Implemented

#### 1. Format Checking
- **Command:** `cargo fmt --check`
- **Purpose:** Enforce consistent code style
- **Action on failure:** Developer runs `cargo fmt` to auto-fix

#### 2. Linting
- **Command:** `cargo clippy --all-targets -- -D warnings`
- **Purpose:** Catch common mistakes and enforce best practices
- **Scope:** All targets (lib, bins, tests, benches)

#### 3. Testing
- **Command:** `cargo test`
- **Purpose:** Verify functionality
- **Coverage:** 59 unit tests across all modules

#### 4. Build Verification
- **Command:** `cargo build --all-targets`
- **Purpose:** Ensure all code (including tests/benches) compiles
- **Prevents:** "Tests don't compile" issues

#### 5. Security Audit
- **Command:** `cargo audit --deny unsound --deny yanked`
- **Purpose:** Detect known vulnerabilities
- **Policy:** Block unsound/yanked crates, allow unmaintained warnings
- **Rationale:** Some warnings (bincode, paste, lru) are low-risk or transitive

## CI Pipeline Structure

### Job 1: Quick Checks (PR + Main)
Runs on all PRs and pushes to main:
1. Checkout code
2. Setup Rust toolchain
3. Enable rust-cache
4. Run format check
5. Run clippy
6. Run tests
7. Build all targets
8. Run security audit

**Fast fail:** Pipeline stops at first failure

### Job 2: Release Builds (Main only)
- Multi-platform builds (Linux, macOS x86/ARM, Windows)
- Only runs on main branch pushes
- Depends on Quick Checks passing

### Job 3: Release Creation (Main only)
- Creates GitHub releases
- Uploads artifacts
- Depends on Release Builds

## Local Development Workflow

Developers can run all checks locally:

```bash
# Quick validation before pushing
cargo fmt --check && \
cargo clippy --all-targets -- -D warnings && \
cargo test && \
cargo build --all-targets && \
cargo audit
```

Or auto-fix formatting:
```bash
cargo fmt
```

## Implementation Changes

### Code Fixes Applied
1. **Formatting:** Fixed all rustfmt violations
2. **Test isolation:** Fixed `test_load_nonexistent` by using unique temp directories per test
   - Added `SaveManager::new_for_test()` method
   - Uses atomic counter for unique test directories
   - Prevents test interference

### CI Workflow Updates
Update `.github/workflows/ci.yml`:
- Add `cargo fmt --check` step
- Update clippy to use `--all-targets` flag
- Add `cargo build --all-targets` step
- Add `cargo audit` step with appropriate flags

## Branch Protection (Recommended)

Configure on GitHub:
1. Go to Settings → Branches
2. Add branch protection rule for `main`
3. Enable "Require status checks to pass before merging"
4. Select "Check & Test" as required check

This prevents merging failing code.

## Future Enhancements

Optional additions for later:
- **Code coverage tracking:** `cargo-tarpaulin` or `cargo-llvm-cov`
- **Documentation checks:** `cargo doc --no-deps`
- **Benchmark regression testing:** Track performance over time
- **Dependabot:** Auto-update dependencies

## Rollback Plan

If checks become too strict:
1. Comment out specific check in ci.yml
2. Create issue to address underlying problem
3. Re-enable check once fixed

## Cost Analysis

- **Public repos:** Free unlimited GitHub Actions minutes
- **Private repos:** 2,000 minutes/month free tier (sufficient)
- **Current usage:** ~2-3 minutes per PR

## Success Metrics

- Zero failing tests merged to main
- Zero formatting inconsistencies
- Zero clippy warnings in production code
- Security vulnerabilities detected before merge
