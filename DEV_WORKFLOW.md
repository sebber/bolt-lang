# Bolt Language - Improved Development Workflow

## 🚀 The Problem We Solved

**Before**: Slow feedback loop, manual testing, inconsistent environments
**After**: Fast feedback, automated CI, easy development setup

## ⚡ Quick Development Commands

### Fast Feedback Loop (choose your preferred tool):

**Option 1: Dev Script** (comprehensive)
```bash
./dev.sh check     # ~10s - Format, lint, compile-check  
./dev.sh test      # ~30s - Full test suite
./dev.sh watch     # Auto-run tests on changes
./dev.sh single hello  # Test one file quickly
```

**Option 2: Just** (modern, clean)
```bash
just check         # Quick development check
just test          # Full test suite
just watch         # File watching
just single hello  # Single test
```

**Option 3: Make** (universal)
```bash
make check         # Development check
make test          # Run tests
make build         # Build binary
```

## 🎯 Typical Development Flow

```bash
# 1. Quick check before coding (10 seconds)
./dev.sh check

# 2. Write code...

# 3. Test specific feature (3 seconds)
./dev.sh single my_test

# 4. Full test when done (30 seconds)
./dev.sh test

# 5. Or watch while coding (auto-runs)
./dev.sh watch
```

## 🔧 One-Time Setup

### New Machine Setup
```bash
# Everything in one command
./setup-dev.sh
```

### Manual Setup  
```bash
# Install Rust components
rustup component add rustfmt clippy

# Install dev tools
cargo install just cargo-watch cargo-audit

# Make scripts executable
chmod +x *.sh

# Initial build
cargo build && ./run_tests.sh
```

## 🧪 Testing Strategy

**2-Tier Testing for Speed**:

1. **Unit Tests** (Rust) - `cargo test` (~5s)
   - Symbol table, parser, lexer tests
   - Run frequently during development

2. **Integration Tests** (Bolt) - `./run_tests.sh` (~30s)  
   - End-to-end compilation pipeline
   - Run before commits

**Individual Test Debugging**:
```bash
./dev.sh single hello      # Run tests/hello.bolt
./dev.sh debug arithmetic  # Show generated C code  
```

## 🤖 CI/CD Pipeline

**GitHub Actions automatically runs**:
- ✅ Code formatting check
- ✅ Clippy linting
- ✅ Unit tests  
- ✅ Integration tests
- ✅ Cross-platform builds
- ✅ Security audit

**Local CI Simulation**:
```bash
./dev.sh check && ./dev.sh test  # Same as CI
```

## 📊 Performance Improvements

| Task | Before | After | Improvement |
|------|--------|--------|-------------|
| Quick Check | Manual | `./dev.sh check` (10s) | Automated |
| Single Test | Compile + Run | `./dev.sh single test` (3s) | 10x faster |
| Full Tests | Manual | `./run_tests.sh` (30s) | Consistent |
| CI Feedback | None | Auto on PR | 🎉 |
| Setup | Complex | `./setup-dev.sh` | 1-command |

## 🛠️ Available Tools

**Core Scripts**:
- `./dev.sh` - Main development script
- `./run_tests.sh` - Integration tests
- `./setup-dev.sh` - Environment setup
- `Justfile` - Modern task runner
- `Makefile` - Universal compatibility

**VS Code Integration**:
```bash
# Install Bolt extension
cd bolt-vscode-extension && ./install.sh

# Pre-configured tasks and debugging
# (see .vscode/ directory)
```

## 🎯 Daily Workflow Examples

**Feature Development**:
```bash
./dev.sh watch          # Start watching files
# Edit code in another terminal
# Tests auto-run on every save
# See immediate feedback
```

**Bug Investigation**:  
```bash
./dev.sh debug failing_test  # See generated C code
./dev.sh single failing_test # Quick test iteration
```

**Pre-Commit**:
```bash
./dev.sh check         # Format, lint, compile-check
./dev.sh test          # Full test suite
git commit -m "..."    # Automatic pre-commit hook
```

## 🚀 Benefits

**For Contributors**:
- ⚡ 10x faster feedback loop  
- 🤖 Automated quality checks
- 🎯 Clear development commands
- 🔧 One-command setup

**For Maintainers**:
- 🛡️ Automatic CI on all PRs
- 📊 Consistent code quality
- 🎯 Easy onboarding process
- 🔄 Cross-platform builds

**For Users**:
- 🚀 Faster development pace
- 🐛 Fewer bugs reaching main
- 📦 Reliable releases
- 🎯 Better tooling ecosystem

---

**Next time you work on Bolt**:
```bash
./dev.sh watch    # Start this, edit code, see instant feedback
```