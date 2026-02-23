---
name: clean-workspace
description: Use when asked to clean up, reset workspace, or make repo look like a fresh clone. Use after finishing feature work, merging PRs, or when workspace has stale branches and uncommitted files.
---

# Clean Workspace

Reset the local repo to a pristine state matching a fresh clone.

## Steps

1. **Stash or warn about uncommitted changes**
   ```bash
   git status -s
   ```
   If there are uncommitted changes, warn the user before proceeding.

2. **Switch to main and pull latest**
   ```bash
   git checkout main && git pull
   ```

3. **Delete merged local branches**
   ```bash
   git branch | grep -v '^\* main$'
   ```
   Delete any listed branches with `git branch -d <branch>`. If a branch has unmerged work, warn instead of force-deleting.

4. **Remove untracked files (if any)**
   Only if untracked files exist and user has confirmed:
   ```bash
   git clean -fd
   ```

5. **Verify clean state**
   ```bash
   git status -s   # Should be empty
   git branch      # Should show only * main
   ```
