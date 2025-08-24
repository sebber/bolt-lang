# Justfile for Bolt Language Development
# Install just: cargo install just
# Run: just <command>

# Default recipe - shows available commands
default:
    @just --list

# Quick development check - formatting, linting, compilation
check:
    @echo "🔍 Running quick development check..."
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings  
    cargo check --all-targets
    @echo "✅ Quick check completed!"

# Auto-fix formatting and linting issues
fix:
    @echo "🔧 Auto-fixing issues..."
    cargo fmt --all
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Run only unit tests (fast)
test-unit:
    @echo "🧪 Running unit tests..."
    cargo test --lib --bins --tests

# Run full test suite (unit + integration)
test:
    @echo "🧪 Running full test suite..."
    cargo build --quiet
    cargo test --quiet
    ./run_tests.sh

# Build debug binary
build:
    @echo "🔨 Building debug binary..."
    cargo build
    @echo "📍 Binary: ./target/debug/bolt"

# Build release binary
build-release:
    @echo "🔨 Building release binary..."
    cargo build --release
    @echo "📍 Binary: ./target/release/bolt"
    @ls -lh target/release/bolt

# Run a single test file
test-single NAME:
    @echo "🧪 Running single test: {{NAME}}"
    cargo build --quiet
    ./target/debug/bolt tests/{{NAME}}.bolt -o test_{{NAME}}
    ./out/debug/test_{{NAME}}

# Debug a test with generated C code
debug NAME:
    @echo "🐛 Debug compiling: {{NAME}}"
    cargo build
    ./target/debug/bolt tests/{{NAME}}.bolt -o debug_{{NAME}}
    @echo "--- Generated C Code ---"
    @cat out/debug/debug_{{NAME}}.c 2>/dev/null || echo "No C file generated"
    @echo "--- Running ---"
    @./out/debug/debug_{{NAME}} 2>/dev/null || echo "Execution failed"

# Clean all build artifacts  
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean
    rm -rf out/debug/* out/release/* out/test/*

# Run all examples
examples:
    @echo "📚 Running examples..."
    ./run_examples.sh

# Performance benchmarks
bench:
    @echo "📊 Running benchmarks..."
    just build-release
    @echo "=== Compilation Time ==="
    time ./target/release/bolt tests/hello.bolt -o bench_hello
    time ./target/release/bolt examples/calculator.bolt -o bench_calc
    @echo "=== Test Suite Time ==="
    time ./run_tests.sh

# Watch files and auto-run tests
watch:
    @echo "👀 Starting file watcher..."
    cargo watch -x check -x test -s './run_tests.sh'

# Check for dependency updates
deps:
    @echo "📦 Checking dependencies..."
    cargo outdated
    cargo audit

# Setup development environment
setup:
    @echo "⚙️ Setting up development environment..."
    rustup component add rustfmt clippy
    cargo install cargo-watch cargo-audit cargo-outdated just
    chmod +x run_tests.sh build.sh dev.sh
    @echo "✅ Setup complete! Try: just check"

# Install VS Code extension locally
install-vscode:
    @echo "📦 Installing VS Code extension..."
    cd bolt-vscode-extension && npm install && ./install.sh

# Lint only (alias for check)
lint: check

# Format code
fmt:
    cargo fmt --all

# Profile a test compilation
profile NAME:
    @echo "📈 Profiling compilation of {{NAME}}"
    perf record -g ./target/release/bolt tests/{{NAME}}.bolt -o profile_{{NAME}}
    perf report

# Count lines of code
loc:
    @echo "📏 Lines of code:"
    @find src -name "*.rs" -exec wc -l {} + | tail -1
    @echo "Tests:"
    @find tests -name "*.bolt" -exec wc -l {} + | tail -1

# Generate documentation
docs:
    @echo "📖 Generating documentation..."
    cargo doc --open

# Security audit
audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Quick commit helper
commit MESSAGE:
    @echo "💾 Quick commit: {{MESSAGE}}"
    git add -A
    git status
    @echo "Press Enter to commit or Ctrl+C to cancel..."
    @read
    git commit -m "{{MESSAGE}}"

# Push to remote
push:
    git push origin main

# Development shell aliases
alias c := check
alias t := test
alias b := build
alias f := fix
alias w := watch
alias e := examples