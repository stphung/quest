#!/bin/bash
# CI checks script - used by both local development and GitHub Actions
# This ensures local and CI run EXACTLY the same checks

set -e  # Exit on first error

echo "🔍 Running CI checks..."
echo ""

# Install cargo-audit if not present (needed for CI, harmless locally)
if ! command -v cargo-audit &> /dev/null; then
    echo "📦 Installing cargo-audit..."
    cargo install cargo-audit --quiet
    echo ""
fi

echo "📝 1/5 Checking formatting..."
cargo fmt --check
echo "✅ Format check passed"
echo ""

echo "🔎 2/5 Running clippy..."
cargo clippy --all-targets --quiet -- -D warnings
echo "✅ Clippy passed"
echo ""

echo "🧪 3/5 Running tests..."
cargo test --quiet
# Screenshot-pipeline converter (scripts/screenshot.sh dependency)
if command -v python3 &> /dev/null; then
    python3 "$(dirname "$0")/ansi2html.py" --self-test
fi
# Act 2 flag-ON smoke tests: the flag_on_* tests self-skip while
# dark-shipped, so this fresh process (OnceLock caches the flag ON) is
# what actually runs them. Mirrored in .github/workflows/ci.yml (test job).
QUEST_ACT2=1 cargo test --quiet flag_on
echo "✅ Tests passed"
echo ""

echo "🎮 4/5 Running progression check (simulator)..."
cargo run --release --quiet --bin simulator -- --check-progression
# Act 2 voyage balance gate: every strategy reaches the Tree inside the
# pacing envelope. Mirrored in .github/workflows/ci.yml (balance job).
cargo run --release --quiet --bin voyage_simulator
echo "✅ Progression check passed"
echo ""

echo "🔒 5/5 Running security audit..."
cargo audit --deny yanked --quiet
echo "✅ Audit passed"
echo ""

# Coverage check (requires cargo-llvm-cov — installed in CI, optional locally)
if command -v cargo-llvm-cov &> /dev/null; then
    echo "📊 5/5 Checking game logic coverage (≥90% lines)..."
    cargo llvm-cov --lib --summary-only --quiet \
        --ignore-filename-regex "(ui/|utils/updater|utils/build_info|tick_events|history/|deep/persistence|enhancement/persistence)" \
        --fail-under-lines 90
    echo "✅ Coverage check passed"
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All CI checks passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
