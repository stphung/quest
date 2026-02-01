# Contributing to Quest

## Commit Message Format

This project uses **Conventional Commits** to automatically generate releases and changelogs.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

Use these types to trigger automatic version bumps:

| Type | Version Bump | Description | Example |
|------|-------------|-------------|---------|
| `feat` | **Minor** (0.X.0) | New feature | `feat: add dragon boss enemy` |
| `fix` | **Patch** (0.0.X) | Bug fix | `fix: correct HP regen timing` |
| `BREAKING CHANGE` | **Major** (X.0.0) | Breaking change | See below |
| `docs` | None | Documentation only | `docs: update README install instructions` |
| `style` | None | Code style/formatting | `style: apply rustfmt` |
| `refactor` | None | Code refactoring | `refactor: simplify combat logic` |
| `test` | None | Add/update tests | `test: add prestige edge cases` |
| `chore` | None | Maintenance | `chore: update dependencies` |
| `ci` | None | CI/CD changes | `ci: add release automation` |

### Scopes (Optional)

Add a scope for context:
- `combat` - Combat system changes
- `ui` - User interface changes
- `prestige` - Prestige system changes
- `save` - Save/load system changes
- `ci` - CI/CD changes

**Examples:**
```bash
feat(combat): add critical hit visual effects
fix(ui): correct zone name rendering
docs(readme): add installation instructions
```

### Breaking Changes

For breaking changes, add `BREAKING CHANGE:` in the footer or use `!` after type:

```bash
feat!: redesign prestige system

BREAKING CHANGE: Prestige ranks now start at 0 instead of 1.
This requires migrating existing save files.
```

## How Releases Work

1. **You commit with conventional format:**
   ```bash
   git commit -m "feat: add new zone type"
   git push
   ```

2. **Release Please analyzes commits:**
   - Detects `feat:` → triggers minor version bump
   - Creates a "Release PR" with:
     - Updated version in `Cargo.toml`
     - Updated `CHANGELOG.md`
     - All changes since last release

3. **You merge the Release PR:**
   - Merging triggers the release
   - Tag is created automatically (e.g., `v0.2.0`)
   - Binaries are built for all platforms
   - GitHub release is published

4. **Users get new version:**
   - One-line installer pulls latest release
   - Release notes are auto-generated from commits

## Development Workflow

### Before Pushing

Run local checks to ensure code quality:

```bash
make check    # Run all CI checks (format, lint, test, build, audit)
make fmt      # Auto-fix formatting issues
```

### Creating a Release

**No manual steps needed!** Just commit with conventional format:

```bash
# Make changes
git add .
git commit -m "feat: add multiplayer support"
git push

# Release Please creates a PR
# Review and merge the PR
# Release happens automatically!
```

### Quick Reference

**New feature (minor version):**
```bash
git commit -m "feat: add achievement system"
```

**Bug fix (patch version):**
```bash
git commit -m "fix: resolve save corruption on crash"
```

**Breaking change (major version):**
```bash
git commit -m "feat!: redesign combat system

BREAKING CHANGE: Combat damage calculation changed.
Old save files need migration."
```

**Documentation (no version change):**
```bash
git commit -m "docs: improve installation guide"
```

## PR Guidelines

1. **Run `make check`** before creating PR
2. **Use conventional commit format** in PR title or commits
3. **All checks must pass** before merge
4. **Squash and merge** is preferred to keep history clean

## Questions?

- Conventional Commits: https://www.conventionalcommits.org/
- Release Please: https://github.com/googleapis/release-please
