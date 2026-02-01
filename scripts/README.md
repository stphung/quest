# CI Scripts

## ci-checks.sh

**The single source of truth for all quality checks.**

This script is used by both:
- **Local development** (via `make check`)
- **GitHub Actions CI** (in `.github/workflows/ci.yml`)

### Why One Script?

Having a single script ensures:
- ✅ Local and CI run **identical** checks
- ✅ No duplication or drift between environments
- ✅ Easy to add/remove/modify checks in one place
- ✅ Developers see exact same results as CI

### Checks Performed

1. **Format** - `cargo fmt --check`
2. **Clippy** - `cargo clippy --all-targets -- -D warnings`
3. **Tests** - `cargo test`
4. **Build** - `cargo build --all-targets`
5. **Security** - `cargo audit --deny yanked`

### Usage

**Locally:**
```bash
make check           # Easiest
./scripts/ci-checks.sh  # Direct
```

**In CI:**
Automatically runs on every PR and push to main.

### Modifying Checks

To add/remove/change checks, edit `scripts/ci-checks.sh` only.
Both local and CI will automatically use the new version.
