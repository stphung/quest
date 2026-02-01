# Development helpers for Quest

.PHONY: check fmt lint test build audit all clean

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

# Just security audit
audit:
	@cargo audit --deny yanked

# Run the game
run:
	@cargo run

# Clean build artifacts
clean:
	@cargo clean

# Default target
all: check
