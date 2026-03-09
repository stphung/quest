---
name: audit
description: Run all audit skills — perf-audit, test-audit, doc-audit, wiki-audit — in sequence. Use when you want a full project health check or before a release.
---

# Full Audit

Run all 4 audit skills in sequence for a comprehensive project health check.

## When to Use

- Before a release
- After landing a large feature
- Periodic health check
- When asked to "audit everything" or "full audit"

## Process

Run each audit skill in order. Each skill creates its own branch, PR, and merges independently via `/ship`.

### Step 1: Performance Audit

Invoke the `perf-audit` skill. Wait for it to complete and merge.

### Step 2: Test Audit

Invoke the `test-audit` skill. Wait for it to complete and merge.

### Step 3: Documentation Audit

Invoke the `doc-audit` skill. Wait for it to complete and merge.

### Step 4: Wiki Audit

Invoke the `wiki-audit` skill. Wait for it to complete and merge.

## Output

Report a summary of all 4 audit results when done.
