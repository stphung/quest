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
echo "✅ Tests passed"
echo ""

echo "🔨 4/5 Building all targets..."
cargo build --all-targets --quiet
echo "✅ Build passed"
echo ""

echo "🔒 5/5 Running security audit..."
cargo audit --deny yanked --quiet
echo "✅ Audit passed"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All CI checks passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
