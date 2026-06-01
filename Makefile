# Aimon - Makefile
# Automated build, format, check and release management

.PHONY: help build release test clean install uninstall \
        fmt check clippy lint dev quick all

# Default target
help:
	@echo "🚀 Aimon - Build System"
	@echo "=============================="
	@echo ""
	@echo "Development:"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  check          - Check code for errors (cargo check)"
	@echo "  clippy         - Run clippy lints"
	@echo "  lint           - Run full linting (fmt + check + clippy)"
	@echo "  dev            - Quick dev cycle (fmt + check + build)"
	@echo "  quick          - Super fast (fmt + check only)"
	@echo ""
	@echo "Building:"
	@echo "  build          - Build debug version"
	@echo "  release        - Build optimized release version"
	@echo "  test           - Run tests"
	@echo "  test-release   - Test the release binary"
	@echo ""
	@echo "Maintenance:"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install locally (requires sudo)"
	@echo "  uninstall      - Uninstall from system"
	@echo "  package        - Create distribution package"
	@echo "  github-release - Create GitHub release (requires gh CLI)"
	@echo "  all            - Full pipeline (lint + test + release)"
	@echo ""
	@echo "Examples:"
	@echo "  make quick           # Fast check before commit"
	@echo "  make dev             # Development cycle"
	@echo "  make lint            # Full quality check"
	@echo "  make release         # Build production binary"

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	@echo "✅ Code formatted"

# Check compilation (fast)
check:
	@echo "🔍 Checking compilation..."
	cargo check 2>&1
	@echo "✅ Compilation check passed"

# Run clippy lints
clippy:
	@echo "🔎 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings 2>&1
	@echo "✅ Clippy passed"

# Full linting pipeline (MUST PASS)
lint: fmt check clippy
	@echo "✨ All linting passed!"

# Quick development check (format + compile check)
quick: fmt check
	@echo "⚡ Quick check complete!"

# Development cycle
dev: fmt check build
	@echo "🔄 Dev cycle complete!"

# Build debug version
build:
	@echo "🔨 Building debug version..."
	cargo build 2>&1
	@echo "✅ Debug build complete"

# Build release version
release:
	@echo "🔨 Building release version..."
	cargo build --release 2>&1
	@echo "✅ Release build complete"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test 2>&1
	@echo "✅ Tests passed"

# Test release binary
test-release: release
	@echo "🧪 Testing release binary..."
	./scripts/test-release.sh

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf release-builds dist
	@echo "✅ Clean complete"

# Install locally
install: release
	@echo "📦 Installing locally..."
	sudo cp target/release/aimon /usr/local/bin/
	sudo chmod +x /usr/local/bin/aimon
	@echo "✅ Installed to /usr/local/bin/aimon"
	@echo "🎯 Run with: aimon"

# Uninstall from system
uninstall:
	@echo "🗑️  Uninstalling..."
	sudo rm -f /usr/local/bin/aimon
	@echo "✅ Uninstalled successfully"

# Create distribution package
package: release test-release
	@echo "📦 Creating distribution package..."
	./scripts/build-release.sh

# Create GitHub release
github-release: package
	@echo "🐙 Creating GitHub release..."
	./scripts/github-release.sh

# Full pipeline (lint + test + release)
all: lint test release
	@echo "✨ All tasks completed successfully!"

# Show version info
version:
	@echo "📊 Version Information:"
	@grep '^version' Cargo.toml
	@echo "Git commit: $$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
	@echo "Build date: $$(date)"

# Check dependencies
check-deps:
	@echo "🔍 Checking dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo not installed"; exit 1; }
	@command -v rustfmt >/dev/null 2>&1 || { echo "❌ rustfmt not installed"; exit 1; }
	@command -v cargo-clippy >/dev/null 2>&1 || { echo "❌ clippy not installed"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "❌ Git not installed"; exit 1; }
	@echo "✅ Dependencies OK"

# Run application (debug)
run: build
	@echo "🚀 Running debug version..."
	./target/debug/aimon

# Run application (release)
run-release: release
	@echo "🚀 Running release version..."
	./target/release/aimon

# Watch mode (requires cargo-watch)
watch:
	@echo "👀 Watching for changes..."
	cargo watch -x "quick"
