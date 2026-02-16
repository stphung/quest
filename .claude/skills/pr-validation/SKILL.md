---
name: pr-validation
description: Create PRs and ensure all CI checks pass before marking ready for review
---

# PR Validation Workflow

## Overview

Automate PR creation with CI validation loop - never leave PRs with failing checks.

**Core principle:** Create → Validate → Fix → Repeat until green.

**Announce at start:** "I'm using the pr-validation skill to create and validate the PR."

## When to Use

- After completing feature/fix work on a branch
- When ready to open a PR for review
- When a PR has failing CI checks that need fixing

## The Process

### Step 1: Create Feature Branch (if not already done)

```bash
# Check current branch
git branch --show-current

# If on main, create feature branch
git checkout -b <type>/<description>
```

**Branch naming:**
- `fix/` - Bug fixes
- `feat/` - New features
- `refactor/` - Code refactoring
- `docs/` - Documentation only
- `style/` - Formatting/style changes

### Step 2: Push Branch and Create PR

```bash
# Push branch
git push -u origin <branch-name>

# Create PR with detailed body
gh pr create --base main --head <branch-name> \
  --title "<type>: <description>" \
  --body "$(cat <<'EOF'
## Summary
- [Bullet points of what changed]

## Problem
[What issue this solves]

## Solution
[How it solves it]

## Testing
[How to verify it works]

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Capture PR number** from output (e.g., PR #9).

### Step 3: Monitor CI Status

```bash
# Check CI status
gh pr checks <pr-number>
```

**Possible states:**
- `pass` - All checks passed, PR is ready ✅
- `pending` - Checks still running, wait and re-check
- `fail` - One or more checks failed, proceed to Step 4

**If pending:** Wait 10-20 seconds and check again.

**If all pass:** Report success and exit.

**If any fail:** Proceed to Step 4.

### Step 4: Identify and Fix Failures

```bash
# Get detailed failure logs
gh run view <run-id> --log-failed
```

**Common failures and fixes:**

#### Format Failure
```
Error: cargo fmt --check failed
```

**Fix:**
```bash
cargo fmt
git add -A
git commit -m "style: apply cargo fmt formatting

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
git push
```

#### Clippy Failure
```
Error: cargo clippy found warnings
```

**Fix:**
1. Read the warnings from logs
2. Fix each warning in the code
3. Run `cargo clippy --all-targets -- -D warnings` locally to verify
4. Commit and push fixes

#### Test Failure
```
Error: cargo test failed
```

**Fix:**
1. Read test failures from logs
2. Fix the failing tests or code
3. Run `cargo test` locally to verify
4. Commit and push fixes

#### Build Failure
```
Error: cargo build failed
```

**Fix:**
1. Read compilation errors from logs
2. Fix the errors
3. Run `cargo build` locally to verify
4. Commit and push fixes

### Step 5: Re-validate After Fixes

```bash
# Push fixes
git push

# Wait for new CI run to start (10-20 seconds)
sleep 15

# Check status again
gh pr checks <pr-number>
```

**Loop:** Repeat Steps 3-5 until all checks pass.

### Step 6: Report Success

When all checks pass:

```
✅ All CI checks passed for PR #<number>

<PR URL>

The PR is ready for review/merge.
```

## Quick Reference

| CI Status | Action |
|-----------|--------|
| `pass` | Report success, done ✅ |
| `pending` | Wait 10-20s, re-check |
| `fail` (format) | Run `cargo fmt`, commit, push |
| `fail` (clippy) | Fix warnings, commit, push |
| `fail` (test) | Fix tests, commit, push |
| `fail` (build) | Fix errors, commit, push |

## Common Mistakes

**Creating PR without branch push:**
- **Problem:** Can't create PR if branch isn't on remote
- **Fix:** Always push branch first with `-u origin <branch>`

**Not waiting for CI to complete:**
- **Problem:** Check status too early, get false "pending"
- **Fix:** Wait 10-20 seconds after push before checking

**Fixing locally but not pushing:**
- **Problem:** Fix applied locally but CI still fails
- **Fix:** Always push after committing fixes

**Not re-checking after fixes:**
- **Problem:** Assume fix worked without validating
- **Fix:** Always re-check CI status after pushing fixes

**Manual fixes without commits:**
- **Problem:** Changes not tracked, can't push
- **Fix:** Always commit fixes with descriptive messages

## Red Flags

**Never:**
- Create PR without validating CI
- Leave PR with failing checks
- Skip re-validation after fixes
- Proceed to review/merge with failing CI
- Force-push to PR branch (loses history)

**Always:**
- Wait for CI to complete before declaring success
- Fix failures immediately when detected
- Push fixes and re-validate
- Include co-authorship in commit messages
- Report final CI status to user

## Integration

**Called after:**
- Feature/fix implementation complete
- Tests passing locally
- Code ready for review

**Pairs with:**
- `superpowers:finishing-a-development-branch` - Sets up the PR creation
- `make check` - Local validation before PR

**Workflow:**
1. Complete feature work
2. Run `make check` locally
3. Use this skill to create PR and validate CI
4. When CI passes, request review or merge

## Example Workflow

```
You: I'm using the pr-validation skill to create and validate the PR.

[Step 1: Check branch - already on fix/character-input-rendering]
[Step 2: Push and create PR]

gh pr create --base main --head fix/character-input-rendering \
  --title "fix: character input text not visible" \
  --body "..."

Created PR #9: https://github.com/user/repo/pull/9

[Step 3: Monitor CI]
gh pr checks 9
→ Check & Test: fail

[Step 4: Identify failure]
gh run view 12345 --log-failed
→ Format failure: cargo fmt --check failed

[Step 4: Fix]
cargo fmt
git add -A
git commit -m "style: apply cargo fmt"
git push

[Step 5: Re-validate]
sleep 15
gh pr checks 9
→ Check & Test: pass

[Step 6: Report]
✅ All CI checks passed for PR #9
https://github.com/user/repo/pull/9
The PR is ready for review/merge.
```
