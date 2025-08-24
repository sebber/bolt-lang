# 🧪 Bolt Language Test Suite

This directory contains comprehensive tests for all Bolt language features.

## 📁 Current Test Structure

### Core Language Tests
- `hello.bolt` - Basic program compilation and execution
- `minimal.bolt` - Minimal valid Bolt program
- `simple_test.bolt` - Basic language constructs
- `test.bolt` - Variable declarations and basic features
- `test_debug.bolt` - Debug output testing

### Data Types & Variables  
- `bool_test.bolt` - Boolean literals and logic
- `colon_test.bolt` - Type annotation syntax
- `array_test.bolt` - Array literals `[1, 2, 3]`
- `array_access_test.bolt` - Array indexing `arr[0]`

### Arithmetic & Logic
- `arithmetic_test.bolt` - Math operations `+, -, *, /, %`
- `comparison_test.bolt` - Comparison operators `==, !=, <, >, <=, >=`
- `logical_test.bolt` - Boolean operators `&&, ||, !`

### Control Flow
- `if_test.bolt` - Basic if statements
- `else_if_test.bolt` - Chained conditionals
- `for_in_test.bolt` - For-in loop iteration
- `simple_for_in_test.bolt` - Basic iteration patterns
- `var_for_in_test.bolt` - Mutable variables in loops
- `debug_for_in_test.bolt` - Loop debugging
- `while_test.bolt` - While loop constructs ⚠️ *Currently failing*
- `for_while_test.bolt` - Mixed loop patterns

### Functions
- `function_with_params.bolt` - Function parameters and calls
- `simple_function.bolt` - Basic function definitions
- `void_function.bolt` - Functions without return values

### Data Structures
- `struct_literal_test.bolt` - Struct creation `Person { name: "Alice" }`
- `struct_access_test.bolt` - Field access `person.name`
- `typedef_test.bolt` - Custom type definitions `def Point = { x: Integer }`

### Module System
- `import_test.bolt` - Basic import functionality
- `import_only.bolt` - Import-only programs
- `simple_import.bolt` - Simple import patterns
- `module_test.bolt` - Full module system test
- `namespace_test.bolt` - Namespace imports `import math from "bolt:math"`
- `namespace_dot_test.bolt` - Dot notation `math.max()`

### Standard Library
- `stdio_test.bolt` - Input/output functions
- `stdlib_test.bolt` - Full standard library test
- `stdlib_namespace.bolt` - Stdlib namespace usage
- `simple_stdlib.bolt` - Basic stdlib functions

### Advanced Features
- `pointer_test.bolt` - Pointer operations `&x`, `^ptr`

## 🎯 Test Categories by Feature

### ✅ **Fully Working (35/37 tests)**
All tests pass except `while_test.bolt` and `for_while_test.bolt` needs expected output.

### 📊 **Test Coverage**

| Feature Category | Tests | Status |
|-----------------|-------|--------|
| **Core Language** | 5 | ✅ 100% |
| **Data Types** | 4 | ✅ 100% |  
| **Arithmetic** | 3 | ✅ 100% |
| **Control Flow** | 7 | ⚠️ 85% (while loops) |
| **Functions** | 3 | ✅ 100% |
| **Data Structures** | 3 | ✅ 100% |
| **Modules** | 6 | ✅ 100% |
| **Standard Library** | 4 | ✅ 100% |
| **Advanced** | 1 | ✅ 100% |

## 🚀 Running Tests

### All Tests
```bash
./run_tests.sh
```

### Single Test
```bash
./target/debug/bolt tests/hello.bolt -o test_hello
./out/debug/test_hello
```

### Adding New Tests

1. **Create test file**: `tests/your_feature_test.bolt`
2. **Run the test**: `./target/debug/bolt tests/your_feature_test.bolt -o your_test`
3. **Capture expected output**: `./out/debug/your_test > tests/expected/your_feature_test.txt`
4. **Verify**: `./run_tests.sh` should show your test passing

## 🎨 Test Writing Guidelines

### Good Test Structure
```bolt
// Test: Feature description
import { print } from "bolt:stdio"

// Setup
val input := 42

// Action  
val result := someOperation(input)

// Verification
print(result)
```

### Test Naming
- `feature_test.bolt` - Main feature test
- `simple_feature.bolt` - Basic/minimal version
- `debug_feature.bolt` - Debug/verbose version

### Expected Output
- Keep output minimal but meaningful
- Use consistent formatting
- Include verification messages where helpful

## 🔧 Future Test Organization

For v0.2.0, consider organizing into subdirectories:
```
tests/
├── core/           # Basic language features
├── data_types/     # Types, variables, arrays  
├── control_flow/   # if/else, loops, flow control
├── functions/      # Function definitions and calls
├── modules/        # Import/export system
├── stdlib/         # Standard library functionality
└── advanced/       # Pointers, complex features
```

## 📈 Test Quality Metrics

- **Coverage**: 35/37 tests passing (94.6%)
- **Categories**: All major language features covered
- **Automation**: Full automated test runner
- **Documentation**: Each test has clear purpose
- **Isolation**: Tests are independent and atomic