# Makefile for mat - Martial Art Tool

.PHONY: help build release test clean install

# Default target
help:
	@echo "mat - Martial Art Tool Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make build       - Build debug binary"
	@echo "  make release     - Build optimized release binary"
	@echo "  make test        - Run all tests"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make install     - Install mat to ~/.local/bin"
	@echo "  make check       - Run tests and linter"

# Build debug binary
build:
	@echo "Building debug binary..."
	cargo build
	@echo "✓ Debug binary: target/debug/mat"

# Build optimized release binary
release:
	@echo "Building optimized release binary..."
	cargo build --release
	@echo "✓ Release binary: target/release/mat"
	@ls -lh target/release/mat | awk '{print "  Size:", $$5}'

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Run tests and clippy
check:
	@echo "Running tests..."
	cargo test
	@echo ""
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✓ Clean complete"

# Install to user's local bin
install: release
	@echo "Installing mat to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/mat ~/.local/bin/
	@chmod +x ~/.local/bin/mat
	@echo "✓ Installed to ~/.local/bin/mat"
	@echo ""
	@echo "Add ~/.local/bin to your PATH if not already present:"
	@echo '  export PATH="$$HOME/.local/bin:$$PATH"'

# Create release artifacts for distribution
dist: release
	@echo "Creating release artifacts..."
	@mkdir -p dist
	@cp target/release/mat dist/mat-linux-x86_64
	@cd dist && tar -czf mat-linux-x86_64.tar.gz mat-linux-x86_64
	@ls -lh dist/mat-linux-x86_64.tar.gz | awk '{print "✓ Created:", $$9, "-", $$5}'
	@echo ""
	@echo "Release artifact ready: dist/mat-linux-x86_64.tar.gz"
