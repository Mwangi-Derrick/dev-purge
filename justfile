# Dev-Purge Build Automation
# Run `just --list` to see available commands

# Set shell for cross-platform compatibility
set shell := ["powershell.exe", "-c"]

# Default recipe (build everything)
default: test-rust lint-rust test-zig

# Rust commands
build-rust:
    cd rust; cargo build --release

build-rust-debug:
    cd rust; cargo build

test-rust:
    cd rust; cargo test

test-rust-verbose:
    cd rust; cargo test --verbose

lint-rust:
    cd rust; cargo clippy -- -D warnings
    cd rust; cargo fmt --check

fix-rust:
    cd rust; cargo clippy --fix
    cd rust; cargo fmt

# Zig commands
build-zig:
    cd zig; zig build -Doptimize=ReleaseFast

test-zig:
    cd zig; zig run src/main.zig -- . --dry-run

# Development helpers
clean:
    cd rust; cargo clean
    cd zig; if (Test-Path zig-cache) { Remove-Item -Recurse -Force zig-cache }
    cd zig; if (Test-Path zig-out) { Remove-Item -Recurse -Force zig-out }

# Install locally
install:
    cd rust; cargo install --path . --force

# CI simulation
ci: test-rust lint-rust test-zig build-rust build-zig
    echo "✅ All CI checks passed!"

# Watch mode for development
watch:
    cd rust; cargo watch -x test