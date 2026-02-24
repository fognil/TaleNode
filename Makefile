.PHONY: build release clean test run

# Development build
build:
	cargo build

# Run in development mode
run:
	cargo run

# Run all tests
test:
	cargo test

# Release build (current platform)
release:
	cargo build --release
	@echo "Binary: target/release/talenode"
	@ls -lh target/release/talenode

# Release build for macOS (Universal Binary - requires both targets installed)
release-macos-universal:
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	mkdir -p target/release-universal
	lipo -create \
		target/aarch64-apple-darwin/release/talenode \
		target/x86_64-apple-darwin/release/talenode \
		-output target/release-universal/talenode
	@echo "Universal binary: target/release-universal/talenode"

# Release build for Windows (cross-compile from macOS/Linux)
release-windows:
	cargo build --release --target x86_64-pc-windows-gnu
	@echo "Binary: target/x86_64-pc-windows-gnu/release/talenode.exe"

# Release build for Linux
release-linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	@echo "Binary: target/x86_64-unknown-linux-gnu/release/talenode"

# Clean build artifacts
clean:
	cargo clean
