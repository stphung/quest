#!/bin/bash
# Generate test coverage report using cargo-tarpaulin
# Run with: make coverage

set -e

echo "📊 Generating test coverage report..."
echo ""

# Install cargo-tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --quiet
    echo ""
fi

# Create coverage directory
mkdir -p coverage

# Run tarpaulin with HTML output
# --skip-clean: Don't clean between runs (faster)
# --out Html: Generate HTML report
# --out Lcov: Generate lcov for CI integration
# --exclude-files: Skip generated/build files
# --ignore-tests: Don't count test code in coverage
echo "🧪 Running tests with coverage..."
cargo tarpaulin \
    --skip-clean \
    --out Html \
    --out Lcov \
    --output-dir coverage \
    --exclude-files "target/*" \
    --exclude-files "build.rs" \
    --ignore-tests \
    --timeout 300 \
    2>&1 | tee coverage/tarpaulin.log

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Coverage report generated!"
echo "   HTML: coverage/tarpaulin-report.html"
echo "   LCOV: coverage/lcov.info"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Open HTML report on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo ""
    read -p "Open coverage report in browser? [Y/n] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        open coverage/tarpaulin-report.html
    fi
fi
