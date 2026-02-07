# Development helpers for Quest

.PHONY: check fmt lint test build audit all clean install setup

# Run all PR checks locally (uses same script as CI)
check:
	@./scripts/ci-checks.sh

# Auto-fix formatting
fmt:
	@cargo fmt

# Just run clippy
lint:
	@cargo clippy --all-targets -- -D warnings

# Just run tests
test:
	@cargo test

# Just build
build:
	@cargo build --all-targets

# Build release and install to ~/.local/bin (with macOS codesigning)
install:
	@cargo build --release
	@mkdir -p ~/.local/bin
	@cp target/release/quest ~/.local/bin/quest
	@if [ "$$(uname)" = "Darwin" ]; then \
		codesign -s - -f ~/.local/bin/quest; \
		echo "Installed and signed: ~/.local/bin/quest"; \
	else \
		echo "Installed: ~/.local/bin/quest"; \
	fi

# Just security audit
audit:
	@cargo audit --deny yanked

# Run the game
run:
	@cargo run

# Clean build artifacts
clean:
	@cargo clean

# Set up development environment (git hooks, etc.)
setup:
	@git config core.hooksPath scripts/hooks
	@echo "Git hooks configured. Pre-commit will now run fmt and clippy checks."

# Default target
all: check
