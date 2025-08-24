# Contributing to Bolt Language

We welcome contributions to the Bolt programming language! This document outlines our development workflow and guidelines.

## 🔀 Branching Strategy

### Feature Branch Workflow

We use a **feature branch workflow** with **squash merging** and **conventional commits**:

1. **Main Branch**: Always stable, protected, requires PR reviews
2. **Feature Branches**: Short-lived branches for individual features/fixes
3. **Squash Merge**: All PRs are squash-merged to keep history clean

### Branch Protection Rules

The `main` branch is protected with:
- ✅ **Required CI checks**: All tests must pass
- ✅ **Required reviews**: At least 1 approval needed  
- ✅ **No direct pushes**: All changes via Pull Requests
- ✅ **No force pushes**: History is immutable
- ✅ **Dismiss stale reviews**: Re-approval needed after changes

## 🚀 Development Workflow

### 1. Create Feature Branch

```bash
# Start from latest main
git checkout main
git pull origin main

# Create feature branch with descriptive name
git checkout -b feat/string-interpolation
# or
git checkout -b fix/parser-error-handling
# or  
git checkout -b docs/improve-readme
```

### 2. Develop with Fast Feedback

Use our fast development tools:

```bash
# Quick development check (10s)
./dev.sh check

# Run specific test (3s) 
./dev.sh single hello_world

# Watch mode for continuous testing
./dev.sh watch

# Run full test suite (30s)
./dev.sh test
```

### 3. Make Conventional Commits

Use **conventional commit format**:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` New feature for users
- `fix:` Bug fix for users
- `docs:` Documentation changes
- `style:` Formatting, missing semi-colons, etc
- `refactor:` Code refactoring without functionality change
- `test:` Adding/updating tests
- `chore:` Build process, tooling, dependencies

**Examples:**
```bash
git commit -m "feat(lexer): add support for single-line comments"
git commit -m "fix(parser): handle empty string literals correctly"
git commit -m "docs: add string concatenation examples to README"
git commit -m "test(integration): add comprehensive array indexing tests"
```

### 4. Push and Create Pull Request

```bash
# Push feature branch
git push origin feat/string-interpolation

# Create PR (will open in browser)
gh pr create --title "feat(lexer): add string interpolation support" \
             --body "Implements string interpolation with \${variable} syntax
             
             - Lexer recognizes interpolation tokens
             - Parser builds interpolation AST nodes  
             - Codegen generates proper C string formatting
             - Comprehensive test coverage added
             
             Closes #123"
```

### 5. Code Review Process

- ✅ **Automated Checks**: CI must pass (formatting, linting, tests)
- ✅ **Manual Review**: At least 1 reviewer approval required
- ✅ **Address Feedback**: Push additional commits to address comments
- ✅ **Final Approval**: Re-approval needed if changes made after review

### 6. Squash Merge

Once approved, **squash merge** the PR:
- All feature branch commits are combined into one
- PR title becomes the final commit message (should be conventional)
- Clean linear history maintained on main
- Feature branch is automatically deleted

## 📋 PR Requirements

### Before Creating PR
- ✅ All tests pass: `./run_tests.sh` 
- ✅ Code is formatted: `cargo fmt`
- ✅ No linting errors: `cargo clippy`
- ✅ New features have tests
- ✅ Documentation updated if needed

### PR Title Format
Use conventional commit format for PR titles:
```
feat(parser): add support for array destructuring
fix(codegen): resolve memory leak in string operations
docs(README): add installation instructions for Windows
```

### PR Description Template
```markdown
## Summary
Brief description of what this PR does.

## Changes Made
- Bullet point list of changes
- Include any breaking changes
- Mention new dependencies

## Testing
- [ ] All existing tests pass
- [ ] New tests added for new functionality  
- [ ] Manual testing performed

## Related Issues
Closes #123
Fixes #456
```

## 🧪 Testing Requirements

### Test Coverage
- ✅ **Unit Tests**: Test individual functions/modules
- ✅ **Integration Tests**: Test complete language features
- ✅ **Regression Tests**: Ensure old bugs don't reappear

### Writing Tests
```bash
# Add new .bolt test file
echo 'import { print } from "bolt:stdio"
fun main() { 
    print("test") 
}' > tests/new_feature_test.bolt

# Add expected output
echo "test" > tests/expected/new_feature_test.txt

# Run the specific test
./dev.sh single new_feature_test
```

### Test Naming
- Test files: `feature_name_test.bolt`
- Expected outputs: `tests/expected/feature_name_test.txt`
- Follow existing patterns for consistency

## 🏗️ Code Style

### Rust Code Style
- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions (snake_case for functions/variables)
- Write meaningful comments for complex logic
- Keep functions focused and small

### Bolt Language Style
- Use consistent indentation (4 spaces)
- Clear, descriptive variable names
- Follow existing test file patterns
- Include comments explaining complex test scenarios

## 🔄 Release Process

### Feature Releases
1. Features are merged to `main` via squash merge
2. CI automatically builds cross-platform binaries
3. Version tags are created for stable releases
4. Release notes generated from conventional commits

### Hotfix Process
For urgent fixes:
1. Create `hotfix/description` branch from `main`
2. Same review process as features
3. Fast-track review if critical
4. Immediate release after merge

## 💡 Tips for Contributors

### Getting Started
1. Read `CLAUDE.md` for technical architecture
2. Run `./setup-dev.sh` to configure development environment
3. Start with small fixes to get familiar with codebase
4. Join discussions in Issues and PRs

### Development Best Practices
- Use `./dev.sh` tools for fast feedback
- Write tests before implementing features (TDD)
- Keep PRs focused on single feature/fix
- Update documentation with code changes
- Test on multiple platforms when possible

### Common Pitfalls
- Don't commit directly to `main` (protected)
- Don't force push to feature branches under review
- Don't mix formatting changes with logic changes
- Don't skip writing tests for new features

## 📞 Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Create an Issue with reproduction steps
- **Features**: Open an Issue to discuss before implementing
- **Code Review**: Tag maintainers if review is needed

---

Thank you for contributing to Bolt! 🚀