.PHONY: all build test clippy clippy-fix fmt clean doc release check

# Default target
all: fmt clippy test

# Build the project
build:
	cargo build

# Run tests
test:
	cargo test --all-features

# Run clippy with the same settings as CI
clippy:
	cargo clippy -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::missing_docs_in_private_items \
		-A clippy::missing_const_for_fn

# Run clippy and fix what can be fixed automatically
clippy-fix:
	cargo clippy --fix --allow-staged --allow-dirty

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean

# Build documentation
doc:
	cargo doc --no-deps --open

# Run all checks (what CI does)
check: fmt clippy test doc
	@echo "All checks passed!"

# Prepare for release - run all checks and show version
release: check
	@echo "Current version: $$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
	@echo "Ready to release! Don't forget to:"
	@echo "  1. Update version in Cargo.toml"
	@echo "  2. Commit and push changes"
	@echo "  3. Create GitHub release with tag v$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"

# Show available commands
help:
	@echo "Available commands:"
	@echo "  make          - Run fmt, clippy, and test"
	@echo "  make build    - Build the project"
	@echo "  make test     - Run tests"
	@echo "  make clippy   - Run strict clippy checks (same as CI)"
	@echo "  make fmt      - Format code"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make doc      - Build and open documentation"
	@echo "  make check    - Run all CI checks"
	@echo "  make release  - Prepare for release"