#!/bin/bash

# Bolt Language Development Script
# Provides fast feedback loop for development

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[DEV]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ️${NC} $1"
}

# Show help
show_help() {
    cat << EOF
Bolt Language Development Script

USAGE:
    ./dev.sh <command> [options]

COMMANDS:
    check           Quick check (fmt, clippy, compile-check)
    test            Run all tests (unit + integration)
    quick-test      Run only fast unit tests
    build           Build debug binary
    build-release   Build release binary
    clean           Clean all build artifacts
    bench           Run performance benchmarks
    lint            Run formatting and linting
    fix             Auto-fix formatting and clippy issues
    watch           Watch files and run tests on changes
    deps            Check for dependency updates
    setup           Setup development environment
    examples        Run all examples
    single <test>   Run a single test file
    debug <test>    Debug a test with extra info
    profile <test>  Profile compilation of a test

EXAMPLES:
    ./dev.sh check              # Quick development check
    ./dev.sh test               # Full test suite
    ./dev.sh single hello       # Run tests/hello.bolt
    ./dev.sh debug arithmetic   # Debug arithmetic_test.bolt
    ./dev.sh watch              # Watch and auto-test
    ./dev.sh setup              # First-time setup

EOF
}

# Quick development check
quick_check() {
    print_status "Running quick development check..."
    
    print_status "Checking code formatting..."
    if cargo fmt --all -- --check; then
        print_success "Code formatting is correct"
    else
        print_error "Code formatting needs fixing. Run: ./dev.sh fix"
        return 1
    fi
    
    print_status "Running clippy lints..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "No clippy warnings"
    else
        print_error "Clippy found issues. Run: ./dev.sh fix"
        return 1
    fi
    
    print_status "Checking compilation..."
    if cargo check --all-targets; then
        print_success "Compilation check passed"
    else
        print_error "Compilation failed"
        return 1
    fi
    
    print_success "Quick check completed successfully!"
}

# Run unit tests only
quick_test() {
    print_status "Running unit tests..."
    
    if cargo test --lib --bins --tests --quiet; then
        print_success "Unit tests passed!"
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Full test suite
full_test() {
    print_status "Running full test suite..."
    
    # Build first
    if ! cargo build --quiet; then
        print_error "Build failed"
        return 1
    fi
    
    # Unit tests
    print_status "Running unit tests..."
    if ! cargo test --quiet; then
        print_error "Unit tests failed"
        return 1
    fi
    
    # Integration tests
    print_status "Running Bolt integration tests..."
    if ! ./run_tests.sh; then
        print_error "Integration tests failed"
        return 1
    fi
    
    print_success "All tests passed!"
}

# Build functions
build_debug() {
    print_status "Building debug binary..."
    if cargo build; then
        print_success "Debug build completed"
        print_info "Binary location: ./target/debug/bolt"
    else
        print_error "Debug build failed"
        return 1
    fi
}

build_release() {
    print_status "Building release binary..."
    if cargo build --release; then
        print_success "Release build completed"
        print_info "Binary location: ./target/release/bolt"
        print_info "Binary size: $(ls -lh target/release/bolt | awk '{print $5}')"
    else
        print_error "Release build failed"
        return 1
    fi
}

# Clean build artifacts
clean_all() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf out/debug/* out/release/* out/test/*
    print_success "Clean completed"
}

# Auto-fix issues
auto_fix() {
    print_status "Auto-fixing formatting and clippy issues..."
    
    print_status "Formatting code..."
    cargo fmt --all
    
    print_status "Running clippy fixes..."
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
    
    print_success "Auto-fix completed"
}

# Run a single test
single_test() {
    local test_name="$1"
    if [ -z "$test_name" ]; then
        print_error "Please specify a test name"
        print_info "Example: ./dev.sh single hello"
        return 1
    fi
    
    local test_file="tests/${test_name}.bolt"
    if [ ! -f "$test_file" ]; then
        test_file="tests/${test_name}_test.bolt"
    fi
    
    if [ ! -f "$test_file" ]; then
        print_error "Test file not found: $test_file"
        print_info "Available tests:"
        ls tests/*.bolt | sed 's/tests\///; s/\.bolt$//' | sed 's/^/  /'
        return 1
    fi
    
    print_status "Building compiler..."
    if ! cargo build --quiet; then
        print_error "Build failed"
        return 1
    fi
    
    print_status "Running test: $test_file"
    
    # Compile test
    local output_name="test_single_${test_name}"
    if ! ./target/debug/bolt "$test_file" -o "$output_name"; then
        print_error "Compilation of $test_file failed"
        return 1
    fi
    
    # Run test
    local actual_output
    actual_output=$(timeout 5s ./out/debug/"$output_name" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 124 ]; then
        print_error "Test timed out"
        return 1
    elif [ $exit_code -ne 0 ]; then
        print_error "Runtime error (exit code $exit_code)"
        return 1
    fi
    
    print_success "Test output:"
    echo "$actual_output" | sed 's/^/  /'
    
    # Check against expected output if available
    local expected_file="tests/expected/${test_name}.txt"
    if [ -f "$expected_file" ]; then
        local expected_output
        expected_output=$(cat "$expected_file")
        if [ "$actual_output" = "$expected_output" ]; then
            print_success "Output matches expected result ✓"
        else
            print_warning "Output differs from expected:"
            print_info "Expected:"
            echo "$expected_output" | sed 's/^/    /'
            print_info "Actual:"
            echo "$actual_output" | sed 's/^/    /'
        fi
    else
        print_info "No expected output file found at $expected_file"
    fi
}

# Debug a test with extra information
debug_test() {
    local test_name="$1"
    if [ -z "$test_name" ]; then
        print_error "Please specify a test name"
        return 1
    fi
    
    local test_file="tests/${test_name}.bolt"
    if [ ! -f "$test_file" ]; then
        test_file="tests/${test_name}_test.bolt"
    fi
    
    if [ ! -f "$test_file" ]; then
        print_error "Test file not found: $test_file"
        return 1
    fi
    
    print_status "Building compiler with debug info..."
    if ! cargo build; then
        print_error "Build failed"
        return 1
    fi
    
    print_status "Debug compiling: $test_file"
    print_info "Generated C code:"
    
    local output_name="test_debug_${test_name}"
    ./target/debug/bolt "$test_file" -o "$output_name" || true
    
    if [ -f "out/debug/${output_name}.c" ]; then
        echo "--- Generated C Code ---"
        cat "out/debug/${output_name}.c"
        echo "--- End C Code ---"
    fi
    
    if [ -f "out/debug/${output_name}" ]; then
        print_status "Running with debug output..."
        ./out/debug/"$output_name"
    fi
}

# Watch files for changes
watch_files() {
    print_status "Starting file watcher (Ctrl+C to stop)..."
    print_info "Watching: src/, tests/, examples/"
    
    if ! command -v inotifywait >/dev/null 2>&1; then
        print_warning "inotifywait not found. Install inotify-tools:"
        print_info "  Ubuntu/Debian: sudo apt install inotify-tools"
        print_info "  Arch: sudo pacman -S inotify-tools"
        print_info "  macOS: brew install fswatch (then use fswatch instead)"
        return 1
    fi
    
    while true; do
        inotifywait -q -r -e modify,create,delete src/ tests/ examples/ --exclude '\.git' 2>/dev/null
        echo
        print_status "Change detected, running quick check..."
        if quick_check && quick_test; then
            print_success "All checks passed! $(date)"
        else
            print_error "Checks failed! $(date)"
        fi
        echo "--- Waiting for changes ---"
    done
}

# Run examples
run_examples() {
    print_status "Running examples..."
    if [ -f "./run_examples.sh" ]; then
        ./run_examples.sh
    else
        print_status "Building compiler..."
        cargo build --quiet
        
        print_status "Running example files..."
        for example in examples/*.bolt; do
            if [ -f "$example" ]; then
                local basename=$(basename "$example" .bolt)
                print_info "Running: $basename"
                ./target/debug/bolt "$example" -o "example_$basename"
                if [ -f "out/debug/example_$basename" ]; then
                    timeout 3s ./out/debug/"example_$basename" || print_warning "Example failed or timed out"
                fi
                echo
            fi
        done
    fi
}

# Simple benchmarking
run_benchmarks() {
    print_status "Running performance benchmarks..."
    
    build_release
    
    print_status "Compilation time benchmarks:"
    echo "--- Simple test ---"
    time ./target/release/bolt tests/hello.bolt -o bench_hello
    
    echo "--- Medium complexity ---"  
    time ./target/release/bolt tests/simple_test.bolt -o bench_simple
    
    echo "--- Complex test ---"
    time ./target/release/bolt examples/calculator.bolt -o bench_calc
    
    print_status "Test suite performance:"
    time ./run_tests.sh
    
    print_status "Binary sizes:"
    ls -lh target/release/bolt target/release/bolt-lsp
}

# Setup development environment
setup_dev() {
    print_status "Setting up development environment..."
    
    # Install Rust components
    print_status "Installing Rust components..."
    rustup component add rustfmt clippy
    
    # Install useful development tools
    print_status "Installing development tools..."
    cargo install cargo-watch cargo-audit cargo-outdated 2>/dev/null || print_warning "Some tools already installed"
    
    # Check system dependencies
    print_status "Checking system dependencies..."
    if command -v gcc >/dev/null 2>&1; then
        print_success "GCC found: $(gcc --version | head -1)"
    else
        print_warning "GCC not found. Install with:"
        print_info "  Ubuntu/Debian: sudo apt install gcc"
        print_info "  macOS: xcode-select --install"
    fi
    
    # Make scripts executable
    chmod +x run_tests.sh build.sh dev.sh
    
    print_success "Development environment setup complete!"
    print_info "Try: ./dev.sh check"
}

# Main command dispatcher
case "${1:-help}" in
    "check"|"c")
        quick_check
        ;;
    "test"|"t")
        full_test
        ;;
    "quick-test"|"qt")
        quick_test
        ;;
    "build"|"b")
        build_debug
        ;;
    "build-release"|"br")
        build_release
        ;;
    "clean")
        clean_all
        ;;
    "lint"|"l")
        quick_check
        ;;
    "fix"|"f")
        auto_fix
        ;;
    "watch"|"w")
        watch_files
        ;;
    "examples"|"e")
        run_examples
        ;;
    "single"|"s")
        single_test "$2"
        ;;
    "debug"|"d")
        debug_test "$2"
        ;;
    "bench"|"benchmark")
        run_benchmarks
        ;;
    "setup")
        setup_dev
        ;;
    "help"|"h"|"--help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo
        show_help
        exit 1
        ;;
esac