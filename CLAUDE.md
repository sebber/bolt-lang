# Bolt Language Compiler - Development Instructions

## Project Overview
Bolt is a simple programming language with TypeScript-inspired syntax that compiles to native code via C transpilation.

### Current Features ✅ FULLY IMPLEMENTED
- **Variables**: `var` (mutable) and `val` (immutable) with type inference (`val name := "value"`)
- **Types**: String, Integer, Bool with explicit typing (`val name: String = "value"`)
- **Control Flow**: if/else/else if statements, for-in loops (`for item in array`), while-style loops (`for (condition)`)
- **Functions**: `fun name(params): ReturnType { ... }` with parameters, return values, and void functions
- **Operators**: Full arithmetic (`+, -, *, /, %`), comparison (`==, !=, <, >, <=, >=`), logical (`&&, ||, !`)
- **Data Structures**: Arrays (`[1, 2, 3]`), array indexing (`arr[0]`), custom types (`def Type = { field: Type }`), struct literals
- **Pointers**: Address-of (`&variable`), dereference (`ptr^`), pointer types (`^Integer`)
- **Module System**: Import/export (`import { func } from "module"`), standard library (`bolt:stdio`, `bolt:math`)
- **Standard Library**: Print functions, math operations (max, min, abs)

### Development Roadmap

## Phase 1: Actually Missing Features (High Priority)

### 1. Enhanced String Operations
- **String concatenation** - `"hello" + " world"`
- **String length** - `str.length` property
- **String indexing** - `str[index]` to get characters
- **String slicing** - `str[start:end]` for substrings

### 2. Enhanced Arrays & Collections
- **Array length** - `numbers.length` property
- **Dynamic arrays** - `var list := []Integer{}` with `push()`/`pop()`
- **Array methods** - `append()`, `remove()`, `contains()`

### 3. C-style For Loops
- **Traditional for loops** - `for (i := 0; i < 10; i++) { ... }`
- **While keyword loops** - `while (condition) { ... }` (currently uses `for (condition)`)

### 4. Enhanced Assignment Operators
- **Mutable variable updates** - ✅ DONE `var x = 5; x = 10` works
- **Assignment operators** - `+=, -=, *=, /=, %=` compound assignments

## Phase 2: Advanced Type System (Medium Priority)

### 5. Error Handling
- **Option types** - `Option<Type>` for nullable values
- **Result types** - `Result<T, E>` for error handling  
- **Try/catch mechanisms** - Error propagation and handling

### 6. Advanced Type Features
- **Type unions** - `String | Integer` flexible typing
- **Generic basics** - `Array<T>`, `Option<T>`, `Map<K, V>`
- **Type aliases** - `type UserId = Integer`
- **Pattern matching** - `match value { Integer(n) => ..., String(s) => ... }`

### 7. Advanced Collections
- **Hash maps** - `Map<String, Integer>` 
- **Sets** - `Set<Integer>`
- **Tuples** - `(String, Integer, Bool)`

## Phase 3: Developer Experience (Partially Complete)

### 8. Language Server & Tooling ✅ PARTIALLY IMPLEMENTED
- **Language Server Protocol (LSP)** ✅ DONE - Working hover, completion 
- **VS Code extension** ✅ DONE - Syntax highlighting + LSP integration
- **Hover information** ✅ DONE - Shows variable/function info
- **Auto-completion** ✅ DONE - Basic completion support
- **Error diagnostics** ⚠️ PARTIAL - LSP diagnostics disabled due to parsing issues
- **Go-to-definition** ❌ TODO - Not yet implemented
- **Code formatting** ❌ TODO - `bolt fmt` command needed

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
- **NEVER commit CLAUDE.md** - This file contains development instructions and should remain local only

**Benefits:**
- Preserves working states for easy rollback
- Creates clear development history
- Prevents loss of progress
- Enables easy collaboration and code review

## Compilation Pipeline
Bolt Source (.bolt) → Lexer → Parser → AST → C Code Generator → GCC → Native Executable