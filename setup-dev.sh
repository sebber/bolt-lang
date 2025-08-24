#!/bin/bash

# Bolt Language Development Environment Setup
# Ensures consistent development environment across machines

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

print_header() {
    echo -e "${PURPLE}════════════════════════════════════════${NC}"
    echo -e "${PURPLE} Bolt Language Development Setup${NC}"
    echo -e "${PURPLE}════════════════════════════════════════${NC}"
    echo
}

print_status() {
    echo -e "${BLUE}[SETUP]${NC} $1"
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

check_command() {
    if command -v "$1" >/dev/null 2>&1; then
        print_success "$1 is installed"
        return 0
    else
        print_warning "$1 is not installed"
        return 1
    fi
}

install_rust() {
    print_status "Checking Rust installation..."
    
    if ! command -v rustc >/dev/null 2>&1; then
        print_warning "Rust is not installed"
        print_status "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        print_success "Rust installed successfully"
    else
        local rust_version=$(rustc --version)
        print_success "Rust is already installed: $rust_version"
    fi
    
    # Update to latest stable
    print_status "Ensuring latest stable Rust..."
    rustup update stable
    rustup default stable
    
    # Install required components
    print_status "Installing Rust components..."
    rustup component add rustfmt clippy
    
    print_success "Rust setup completed"
}

install_tools() {
    print_status "Installing development tools..."
    
    local tools=(
        "cargo-watch"    # File watching
        "cargo-audit"    # Security auditing
        "cargo-outdated" # Dependency updates
        "just"          # Task runner
    )
    
    for tool in "${tools[@]}"; do
        print_status "Installing $tool..."
        if cargo install "$tool" 2>/dev/null; then
            print_success "$tool installed"
        else
            print_warning "$tool may already be installed"
        fi
    done
}

check_system_deps() {
    print_status "Checking system dependencies..."
    
    # Detect OS
    local os=""
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        os="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        os="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        os="windows"
    else
        print_warning "Unknown OS type: $OSTYPE"
        os="unknown"
    fi
    
    print_info "Detected OS: $os"
    
    # Check for C compiler
    if check_command gcc || check_command clang || check_command cl; then
        print_success "C compiler found"
    else
        print_error "No C compiler found!"
        case $os in
            "linux")
                print_info "Install with: sudo apt install gcc (Ubuntu/Debian) or sudo pacman -S gcc (Arch)"
                ;;
            "macos")
                print_info "Install with: xcode-select --install"
                ;;
            "windows")
                print_info "Install Visual Studio Build Tools or MSYS2"
                ;;
        esac
        return 1
    fi
    
    # Optional tools
    print_status "Checking optional development tools..."
    
    local optional_tools=(
        "git"
        "code"      # VS Code
        "inotifywait"  # For file watching on Linux
        "fswatch"   # For file watching on macOS
        "perf"      # For profiling
        "valgrind"  # For memory debugging
    )
    
    for tool in "${optional_tools[@]}"; do
        if check_command "$tool"; then
            continue
        else
            case $tool in
                "inotifywait")
                    if [[ "$os" == "linux" ]]; then
                        print_info "Install with: sudo apt install inotify-tools (Ubuntu/Debian)"
                    fi
                    ;;
                "fswatch")
                    if [[ "$os" == "macos" ]]; then
                        print_info "Install with: brew install fswatch"
                    fi
                    ;;
                "code")
                    print_info "Install VS Code from: https://code.visualstudio.com/"
                    ;;
                "perf")
                    if [[ "$os" == "linux" ]]; then
                        print_info "Install with: sudo apt install linux-perf (Ubuntu/Debian)"
                    fi
                    ;;
                "valgrind")
                    if [[ "$os" == "linux" ]]; then
                        print_info "Install with: sudo apt install valgrind"
                    fi
                    ;;
            esac
        fi
    done
}

setup_git_hooks() {
    print_status "Setting up Git hooks..."
    
    if [ ! -d ".git" ]; then
        print_warning "Not a git repository, skipping Git hooks setup"
        return
    fi
    
    mkdir -p .git/hooks
    
    # Pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

echo "Running pre-commit checks..."

# Check formatting
if ! cargo fmt --all -- --check; then
    echo "❌ Code is not formatted. Run: cargo fmt"
    exit 1
fi

# Run clippy
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues. Fix them or run: cargo clippy --fix"
    exit 1
fi

# Run unit tests
if ! cargo test --lib --bins --tests --quiet; then
    echo "❌ Unit tests failed"
    exit 1
fi

echo "✅ Pre-commit checks passed"
EOF

    chmod +x .git/hooks/pre-commit
    print_success "Git pre-commit hook installed"
}

create_vscode_config() {
    print_status "Setting up VS Code configuration..."
    
    mkdir -p .vscode
    
    # Settings
    cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "files.exclude": {
        "**/target": true,
        "**/out": true,
        "**/*.c": false
    },
    "files.associations": {
        "*.bolt": "rust"
    },
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
EOF

    # Tasks
    cat > .vscode/tasks.json << 'EOF'
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "build",
            "type": "shell",
            "command": "cargo build",
            "group": "build",
            "problemMatcher": "$rustc"
        },
        {
            "label": "test",
            "type": "shell", 
            "command": "./run_tests.sh",
            "group": "test"
        },
        {
            "label": "check",
            "type": "shell",
            "command": "./dev.sh check",
            "group": "build"
        },
        {
            "label": "examples",
            "type": "shell",
            "command": "./run_examples.sh"
        }
    ]
}
EOF

    # Launch configuration
    cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug Bolt Compiler",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/bolt",
            "args": ["tests/hello.bolt", "-o", "hello_debug"],
            "cwd": "${workspaceFolder}",
            "sourceLanguages": ["rust"]
        }
    ]
}
EOF

    print_success "VS Code configuration created"
}

make_executable() {
    print_status "Making scripts executable..."
    
    local scripts=(
        "dev.sh"
        "run_tests.sh" 
        "run_examples.sh"
        "build.sh"
        "setup-dev.sh"
    )
    
    for script in "${scripts[@]}"; do
        if [ -f "$script" ]; then
            chmod +x "$script"
            print_success "$script is now executable"
        fi
    done
}

initial_build() {
    print_status "Performing initial build and test..."
    
    print_status "Building debug binary..."
    if cargo build; then
        print_success "Debug build successful"
    else
        print_error "Debug build failed"
        return 1
    fi
    
    print_status "Running unit tests..."
    if cargo test --quiet; then
        print_success "Unit tests passed"
    else
        print_warning "Some unit tests failed"
    fi
    
    print_status "Running integration tests..."
    if ./run_tests.sh >/dev/null 2>&1; then
        print_success "Integration tests passed"
    else
        print_warning "Some integration tests failed"
    fi
}

create_aliases() {
    print_status "Creating helpful aliases..."
    
    cat > .bash_aliases_bolt << 'EOF'
# Bolt Language Development Aliases
alias bolt-check="./dev.sh check"
alias bolt-test="./dev.sh test"
alias bolt-build="./dev.sh build"
alias bolt-clean="./dev.sh clean"
alias bolt-watch="./dev.sh watch"
alias bolt-examples="./dev.sh examples"

# Quick aliases
alias bc="./dev.sh check"
alias bt="./dev.sh test"
alias bb="./dev.sh build"
alias bw="./dev.sh watch"

# Just aliases (if just is installed)
alias j="just"
alias jc="just check"
alias jt="just test"
alias jb="just build"
EOF

    print_success "Aliases created in .bash_aliases_bolt"
    print_info "To use aliases, run: source .bash_aliases_bolt"
}

show_next_steps() {
    echo
    echo -e "${GREEN}🎉 Development environment setup complete!${NC}"
    echo
    echo -e "${CYAN}Next steps:${NC}"
    echo -e "  1. Source aliases:      ${YELLOW}source .bash_aliases_bolt${NC}"
    echo -e "  2. Quick check:         ${YELLOW}./dev.sh check${NC}"
    echo -e "  3. Run tests:           ${YELLOW}./dev.sh test${NC}"
    echo -e "  4. Start watching:      ${YELLOW}./dev.sh watch${NC}"
    echo -e "  5. Or with just:        ${YELLOW}just check${NC}"
    echo
    echo -e "${CYAN}Available commands:${NC}"
    echo -e "  ${YELLOW}./dev.sh help${NC}          - Show all dev commands"
    echo -e "  ${YELLOW}just${NC}                   - Show all just commands"  
    echo -e "  ${YELLOW}cargo test${NC}             - Run unit tests"
    echo -e "  ${YELLOW}./run_tests.sh${NC}         - Run integration tests"
    echo
    echo -e "${CYAN}VS Code:${NC}"
    echo -e "  - Install Bolt extension: ${YELLOW}cd bolt-vscode-extension && ./install.sh${NC}"
    echo -e "  - Recommended extensions: rust-analyzer, Error Lens, GitLens"
    echo
}

# Main setup flow
main() {
    print_header
    
    print_status "Starting development environment setup..."
    echo
    
    install_rust
    echo
    
    check_system_deps
    echo
    
    install_tools
    echo
    
    make_executable
    echo
    
    setup_git_hooks
    echo
    
    create_vscode_config
    echo
    
    create_aliases
    echo
    
    initial_build
    echo
    
    show_next_steps
}

# Check if we're being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi