# Makefile for Bolt Language Development
# Compatible across different systems

.PHONY: help check test test-unit build build-release clean examples install setup watch lint fix debug
.DEFAULT_GOAL := help

# Colors
GREEN := \033[0;32m
YELLOW := \033[1;33m  
BLUE := \033[0;34m
RESET := \033[0m

# Help target
help: ## Show available commands
	@echo "$(BLUE)Bolt Language Development Commands:$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(BLUE)Alternative tools:$(RESET)"
	@echo "  $(YELLOW)./dev.sh$(RESET)          - Development script with more options"
	@echo "  $(YELLOW)just$(RESET)              - Modern task runner (install with: cargo install just)"

# Development commands
check: ## Quick check (format, lint, compile-check)
	@echo "$(GREEN)Running quick development check...$(RESET)"
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo check --all-targets
	@echo "$(GREEN)✅ Quick check completed!$(RESET)"

lint: check ## Alias for check

fix: ## Auto-fix formatting and linting issues
	@echo "$(GREEN)Auto-fixing issues...$(RESET)"
	@cargo fmt --all
	@cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

test-unit: ## Run only unit tests (fast)
	@echo "$(GREEN)Running unit tests...$(RESET)"
	@cargo test --lib --bins --tests

test: ## Run full test suite (unit + integration)
	@echo "$(GREEN)Running full test suite...$(RESET)"
	@cargo build --quiet
	@cargo test --quiet
	@./run_tests.sh

# Build commands
build: ## Build debug binary
	@echo "$(GREEN)Building debug binary...$(RESET)"
	@cargo build
	@echo "$(BLUE)Binary location: ./target/debug/bolt$(RESET)"

build-release: ## Build release binary 
	@echo "$(GREEN)Building release binary...$(RESET)"
	@cargo build --release
	@echo "$(BLUE)Binary location: ./target/release/bolt$(RESET)"
	@ls -lh target/release/bolt

# Utility commands
clean: ## Clean all build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(RESET)"
	@cargo clean
	@rm -rf out/debug/* out/release/* out/test/* 2>/dev/null || true

examples: ## Run all examples
	@echo "$(GREEN)Running examples...$(RESET)"
	@./run_examples.sh

setup: ## Setup development environment
	@echo "$(GREEN)Setting up development environment...$(RESET)"
	@./setup-dev.sh

watch: ## Watch files and auto-run tests (requires cargo-watch)
	@echo "$(GREEN)Starting file watcher...$(RESET)"
	@cargo watch -x check -x test -s './run_tests.sh'

debug: ## Debug build with extra info
	@echo "$(GREEN)Debug build with extra information...$(RESET)"
	@RUST_BACKTRACE=1 cargo build
	@echo "$(BLUE)Debug binary ready with backtrace enabled$(RESET)"

# Individual test running
test-hello: build ## Run hello test
	@./target/debug/bolt tests/hello.bolt -o test_hello
	@./out/debug/test_hello

test-simple: build ## Run simple test
	@./target/debug/bolt tests/simple_test.bolt -o test_simple  
	@./out/debug/test_simple

# Quality assurance
audit: ## Run security audit
	@echo "$(GREEN)Running security audit...$(RESET)"
	@cargo audit

bench: build-release ## Run performance benchmarks
	@echo "$(GREEN)Running benchmarks...$(RESET)"
	@echo "=== Compilation Time ==="
	@time ./target/release/bolt tests/hello.bolt -o bench_hello
	@time ./target/release/bolt examples/calculator.bolt -o bench_calc
	@echo "=== Test Suite Time ==="
	@time ./run_tests.sh
	@echo "=== Binary Sizes ==="
	@ls -lh target/release/bolt target/release/bolt-lsp

# Documentation
docs: ## Generate and open documentation
	@echo "$(GREEN)Generating documentation...$(RESET)"
	@cargo doc --open

# Installation
install-local: build-release ## Install locally built binary
	@echo "$(GREEN)Installing locally built binary...$(RESET)"
	@cp target/release/bolt ~/.local/bin/ 2>/dev/null || cp target/release/bolt /usr/local/bin/
	@cp target/release/bolt-lsp ~/.local/bin/ 2>/dev/null || cp target/release/bolt-lsp /usr/local/bin/
	@echo "$(GREEN)✅ Installed bolt and bolt-lsp$(RESET)"

install-vscode: ## Install VS Code extension
	@echo "$(GREEN)Installing VS Code extension...$(RESET)"
	@cd bolt-vscode-extension && npm install && ./install.sh

# Git helpers
commit: ## Quick commit (make commit m="message")
ifdef m
	@git add -A
	@git status
	@echo "Committing: $(m)"
	@git commit -m "$(m)"
else
	@echo "$(YELLOW)Usage: make commit m=\"Your commit message\"$(RESET)"
endif

push: ## Push to remote
	@git push origin main

# Development shortcuts (common typos/alternatives)
fmt: fix
format: fix
tests: test
t: test
b: build
c: check
e: examples