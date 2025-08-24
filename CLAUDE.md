# Bolt Language Compiler - Development Instructions

## Project Overview
Bolt is a simple programming language with TypeScript-inspired syntax that compiles to native code via C transpilation.

### Current Features
- Variables: `var` (mutable) and `val` (immutable)
- Types: String, Integer, Bool with type inference (`val name := "value"`)
- Control flow: if/else if/else statements
- Functions: `fun name(params): ReturnType { ... }` with calls and return values
- Built-in functions: `print()`

### Development Roadmap

## Phase 1: Essential for Real Programs (High Priority)

### 1. Memory Management & Pointers
- **Odin-style pointer syntax** - `val ptr: ^Integer := &variable`, `val value := ptr^`
- **Address-of operator** - `&variable` to get memory address
- **Dereference operator** - `ptr^` to access pointed value
- **No pointer arithmetic** - Keep it safe like Odin
- **C transpilation**: Maps directly to C pointers (`int*`, `*ptr`, `&var`)

### 2. Arrays & Collections
- **Array literals** - `val numbers := [1, 2, 3, 4, 5]`
- **Array indexing** - `val item := numbers[2]`
- **Array length** - `numbers.length` property
- **Dynamic arrays** - `var list := []Integer{}` with `push()`/`pop()`

### 3. Iteration & Loops
- **For-in loops** - `for (item in items) { ... }`
- **C-style for loops** - `for (i := 0; i < 10; i++) { ... }`
- **While loops** - `while (condition) { ... }`

### 4. String Operations
- **String concatenation** - `"hello" + " world"`
- **String length** - `str.length`
- **String indexing** - `str[index]`
- **String slicing** - `str[start:end]` (future)

## Phase 2: Enhanced Language Features (Medium Priority)

### 5. Error Handling
- **Option types** - `Option<Type>` for nullable values
- **Result types** - `Result<T, E>` for error handling
- **Pattern matching on options** - `match result { ... }`

### 6. Enhanced Operators
- **Arithmetic operators** - `+, -, *, /, %` (partially done)
- **Comparison operators** - `==, !=, <, >, <=, >=` (partially done) 
- **Logical operators** - `&&, ||, !` (partially done)
- **Assignment operators** - `+=, -=, *=, /=`

### 7. Better Type System
- **Type unions** - `String | Integer` flexible typing
- **Generic basics** - `Array<T>`, `Option<T>`
- **Type aliases** - `type UserId = Integer`

## Phase 3: Developer Experience (High Priority)

### 8. Language Server & Tooling
- **Language Server Protocol (LSP)** implementation
- **Syntax highlighting** for VS Code, Vim, etc.
- **Error diagnostics** with line numbers and suggestions
- **Auto-completion** for variables, functions, imports
- **Go-to-definition** and hover information
- **Code formatting** (bolt fmt command)

### 9. Enhanced Error Reporting
- **Better error messages** with context and suggestions
- **Line/column information** in all error types
- **Error recovery** - continue parsing after errors
- **Warning system** - unused variables, deprecated features

## Phase 4: Advanced Features (Lower Priority)

### 10. Advanced Data Structures
- **Hash maps** - `Map<String, Integer>`
- **Sets** - `Set<Integer>`
- **Tuples** - `(String, Integer, Bool)`

### 11. Functional Features (No OOP)
- **Function overloading with pattern matching** - Multiple function definitions with same name
- **Pattern-based dispatch** - Functions chosen by argument patterns at compile time
- **Compile-time code generation** - Like Elixir but statically typed
- **Algebraic data types** - Enums with associated data for rich type modeling

**Example Elixir-style Function Overloading:**
```bolt
// Multiple definitions of same function name
fun process(value: Integer) -> String {
    return "Got integer: " + value.to_string()
}

fun process(value: String) -> String {
    return "Got string: " + value
}

fun process(value: Bool) -> String {
    return "Got boolean: " + (value ? "true" : "false")
}

// Compiler generates dispatch logic based on argument types
// No runtime type checking needed - all resolved at compile time
```

### 12. Advanced Language Features
- **Pattern matching** - `match value { Integer(n) => ..., String(s) => ... }`
- **Guard clauses** - `fun factorial(n: Integer) when n > 0 -> Integer`
- **Async/await** - Concurrent programming
- **Package manager** - Dependency management

## Development Tools Priority

**Immediate Need (Phase 3):**
1. **LSP server** - Essential for developer adoption
2. **VS Code extension** - Syntax highlighting + IntelliSense  
3. **Error diagnostics** - Better debugging experience
4. **Auto-formatting** - Consistent code style

**Supporting Infrastructure:**
- **Documentation generator** - Generate docs from code
- **Test framework** - Built-in testing capabilities
- **Benchmarking tools** - Performance measurement
- **Debugger integration** - GDB integration for compiled output

## Test Case Management

**CRITICAL: The `tests/` directory contains regression test cases that must be preserved.**

### Test Directory Structure
- `tests/` - Contains all test cases for the language
- `tests/*.bolt` - Bolt source files demonstrating language features
- `tests/expected/` - Expected output files for each test

### Test Case Guidelines
1. **NEVER delete test files** unless the language behavior is intentionally changing
2. **NEVER modify existing test files** unless fixing a bug or changing intended behavior
3. **ALWAYS add new test cases** when implementing new features
4. **PRESERVE backward compatibility** - old test cases should continue to work

### Current Test Coverage
- Boolean literals and variables (`bool_test.bolt`)
- Basic if/else statements (`if_test.bolt`) 
- Else if chains (`else_if_test.bolt`)
- Variable declarations with type inference (`test.bolt`)
- Hello world printing (`hello.bolt`)

### When Implementing New Features
1. Create test cases BEFORE implementing the feature
2. Test cases should cover:
   - Happy path scenarios
   - Edge cases
   - Error conditions (when error handling is implemented)
3. Add expected output files to document intended behavior

### Test Execution
**Automated test runner:** `./run_tests.sh`
- Builds the compiler
- Runs all test cases in `tests/*.bolt` in debug mode
- Compares output with expected results in `tests/expected/*.txt`
- Reports pass/fail status
- Test executables and C files are saved to `out/debug/` for debugging

**Manual testing:** 
- Debug build: `./target/debug/bolt <test_file.bolt> -o <output>`
- Release build: `./target/debug/bolt <test_file.bolt> -o <output> --release`

### Build Output Structure
```
out/
├── debug/          # Debug builds with C source files preserved
│   ├── program     # Debug executable with -g flag
│   └── program.c   # Generated C source (kept for debugging)
└── release/        # Optimized builds
    └── program     # Release executable with -O2 optimization
```

**Debug Mode:**
- Shows generated C code
- Includes debug symbols (-g)
- Preserves C source files
- Outputs to `out/debug/`

**Release Mode:**
- Silent compilation
- Optimized with -O2
- Cleans up C source files
- Outputs to `out/release/`

## Architecture Notes
- Lexer: `src/lexer.rs` - Tokenizes Bolt source code
- Parser: `src/parser.rs` - Builds AST from tokens  
- AST: `src/ast.rs` - Language constructs representation
- Code Generation: `src/c_codegen.rs` - Transpiles AST to C code
- Main: `src/main.rs` - CLI interface and compilation pipeline

## Development Workflow
1. Add test cases for new features
2. Update AST if needed
3. Update lexer for new tokens
4. Update parser for new syntax
5. Update code generator
6. Test compilation and execution
7. **ALWAYS run `./run_tests.sh` to ensure all existing tests still pass**
8. Only after all tests pass should changes be considered complete

**CRITICAL:** Before making ANY changes to the language, run the test suite to establish a baseline. After changes, run tests again to ensure no regressions.

## Commit Policy
**COMMIT AS OFTEN AS POSSIBLE** - Whenever you have working code with 100% passing tests, immediately commit and push the changes. This includes:

- After implementing any new feature (even partial)
- After fixing any bugs or issues
- After adding new test cases
- After refactoring code
- After any meaningful progress

**Guidelines:**
- Always run `./run_tests.sh` before committing
- Only commit when ALL tests pass (100% success rate)
- Write descriptive commit messages explaining what was added/fixed
- Push to remote repository immediately after committing
- Never let working progress sit uncommitted for extended periods

**Benefits:**
- Preserves working states for easy rollback
- Creates clear development history
- Prevents loss of progress
- Enables easy collaboration and code review

## Compilation Pipeline
Bolt Source (.bolt) → Lexer → Parser → AST → C Code Generator → GCC → Native Executable