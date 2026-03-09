---
name: ship
description: Push branch, create PR with automerge, and watch until merged. Use when work is done and ready to land — "ship it", "push and merge", "land this".
---

# Ship

Push the current branch, create a squash-merge PR, enable automerge, and watch CI until it merges. Fix any issues that arise.

## Prerequisites

- Current branch is NOT main (must be on a feature/fix branch)
- All changes are committed (no uncommitted work)
- `gh` CLI is authenticated

## Steps

1. **Validate state**
   ```bash
   git status -s          # Must be clean
   git branch --show-current  # Must not be main
   ```
   If on main, abort with a message. If there are uncommitted changes, warn the user.

2. **Push branch**
   ```bash
   git push -u origin HEAD
   ```

3. **Create PR**
   - Use `git log main..HEAD` to understand all commits on the branch
   - Generate a concise PR title (under 70 characters) and body summarizing the changes
   - Create the PR:
     ```bash
     gh pr create --title "..." --body "..."
     ```

4. **Enable automerge**
   ```bash
   gh pr merge <number> --auto --squash
   ```

5. **Watch until merged**
   Use `gh run watch` to monitor the CI run triggered by the PR:
   ```bash
   gh run watch $(gh run list --branch $(git branch --show-current) --limit 1 --json databaseId --jq '.[0].databaseId')
   ```
   This blocks until the run completes — no polling needed.

6. **Verify merge**
   ```bash
   gh pr view <number> --json state,mergedAt
   ```
   Confirm the PR state is `MERGED`.

7. **Handle failures**
   If CI fails:
   - Read the failing check logs: `gh run view <run-id> --log-failed`
   - Diagnose and fix the issue
   - Commit the fix, push, and return to step 5 (watch the new run)
   - Repeat until CI passes and the PR merges

## Output

Report the PR URL and final merge status when done.
