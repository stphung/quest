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

echo "📝 1/4 Checking formatting..."
cargo fmt --check
echo "✅ Format check passed"
echo ""

echo "🔎 2/4 Running clippy..."
cargo clippy --all-targets --quiet -- -D warnings
echo "✅ Clippy passed"
echo ""

echo "🧪 3/4 Running tests..."
cargo test --quiet
echo "✅ Tests passed"
echo ""

echo "🔒 4/4 Running security audit..."
cargo audit --deny yanked --quiet
echo "✅ Audit passed"
echo ""

# Coverage check (requires cargo-llvm-cov — installed in CI, optional locally)
if command -v cargo-llvm-cov &> /dev/null; then
    echo "📊 5/5 Checking game logic coverage (≥90% lines)..."
    cargo llvm-cov --lib --summary-only --quiet \
        --ignore-filename-regex "(ui/|utils/updater|utils/build_info|tick_events)" \
        --fail-under-lines 90
    echo "✅ Coverage check passed"
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All CI checks passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
