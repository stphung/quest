# Development helpers for Quest

# Pin to the CLI version the .claude/skills/openspec-*/SKILL.md files were
# generated against (see each file's `generatedBy` frontmatter) so the
# `/opsx:*` skills' bare `openspec ...` commands behave predictably.
OPENSPEC_VERSION := 1.5.0

.PHONY: check fmt lint test build audit all clean install setup openspec-setup coverage coverage-html coverage-check eval-validate

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

# Format and build
build:
	@cargo fmt
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

# Test coverage summary (requires: cargo install cargo-llvm-cov)
coverage:
	@cargo llvm-cov --lib --summary-only

# Test coverage HTML report (opens in browser)
coverage-html:
	@cargo llvm-cov --lib --html --open

# Enforce ≥90% line coverage on game logic (excludes UI, updater, build_info)
coverage-check:
	@cargo llvm-cov --lib --summary-only --quiet \
		--ignore-filename-regex "(ui/|utils/updater|utils/build_info|tick_events)" \
		--fail-under-lines 90

# Integrity-check the model-eval task suite (red on bug, green on reference).
# Fast tier only; add "--tier all" manually to validate simulator-graded tasks.
eval-validate:
	@python3 evals/harness/run.py validate

# Clean build artifacts
clean:
	@cargo clean

# Set up development environment (git hooks, OpenSpec CLI, etc.)
setup: openspec-setup
	@git config core.hooksPath scripts/hooks
	@echo "Git hooks configured. Pre-commit will now run fmt and clippy checks."

# Install the OpenSpec CLI the /opsx:* skills (.claude/skills/openspec-*)
# and openspec/README.md's workflow depend on. Non-fatal if npm is missing
# or the install fails — the skills just won't work until it's installed
# manually (see openspec/README.md).
openspec-setup:
	@if command -v openspec >/dev/null 2>&1 && [ "$$(openspec --version 2>/dev/null)" = "$(OPENSPEC_VERSION)" ]; then \
		echo "OpenSpec CLI $(OPENSPEC_VERSION) already installed."; \
	elif command -v npm >/dev/null 2>&1; then \
		npm install -g @fission-ai/openspec@$(OPENSPEC_VERSION) \
			&& echo "OpenSpec CLI $(OPENSPEC_VERSION) installed." \
			|| echo "Warning: failed to install the OpenSpec CLI; /opsx:* skills won't work until 'npm install -g @fission-ai/openspec@$(OPENSPEC_VERSION)' succeeds."; \
	else \
		echo "Warning: npm not found; skipping OpenSpec CLI install. /opsx:* skills need 'npm install -g @fission-ai/openspec@$(OPENSPEC_VERSION)' run manually."; \
	fi

# Default target
all: check
