# Bolt Language Examples

This directory contains example programs demonstrating the features of the Bolt programming language.

## Quick Start

```bash
# Run a simple hello world
./target/debug/bolt examples/hello.bolt -o hello
./out/debug/hello

# Run all examples at once  
./run_examples.sh
```

## hello.bolt

The classic "Hello, World!" program demonstrating:
- Import statements: `import { print } from "bolt:stdio"`
- Basic print functionality

```bash
./target/debug/bolt examples/hello.bolt -o hello
./out/debug/hello
```

## calculator.bolt

A comprehensive example showcasing:

- **Variables**: `val` (immutable) and `var` (mutable) declarations
- **Functions**: User-defined functions with parameters and return types
- **Data types**: Integers, booleans, strings, and arrays
- **Arithmetic operations**: `+`, `-`, `*`, `/`, `%` with proper precedence
- **Comparison operations**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Conditional statements**: `if`/`else` branching
- **Type inference**: Automatic type detection with `:=` syntax
- **Arrays**: Array literals with `[1, 2, 3]` syntax

### Running the example

```bash
./target/debug/bolt examples/calculator.bolt -o calculator
./out/debug/calculator
```

### Expected output

The calculator demonstrates various operations and should output mathematical results, boolean comparisons, and complex expressions, showcasing the language's capabilities in a practical context.

## logic_demo.bolt

A demonstration of logical operators and boolean logic featuring:

- **Logical operators**: `&&` (and), `||` (or), `!` (not)
- **Complex boolean expressions**: Combining multiple logical operations
- **Practical scenarios**: Real-world logic like eligibility checks, range validation
- **Conditional logic**: Using logical results in if/else statements

```bash
./target/debug/bolt examples/logic_demo.bolt -o logic_demo
./out/debug/logic_demo
```

## Features Demonstrated

### Variable Declarations
```bolt
val x := 42                    // Immutable integer
val name := "Bolt"             // Immutable string
val numbers := [1, 2, 3]       // Immutable array
var counter := 0               // Mutable integer
```

### Function Definitions
```bolt
fun add(a: Integer, b: Integer): Integer {
    return a + b
}

fun is_even(n: Integer): Bool {
    return n % 2 == 0
}
```

### Expressions and Operations
```bolt
val result := (x + y) * 2 - 10      // Arithmetic with precedence
val comparison := x > y && y < 100  // Boolean operations  
val complex := max(add(x, 5), y)    // Function composition
val logic := !false || (x >= 10 && y != 0)  // Logical operators
```

The Bolt language prioritizes simplicity, type safety, and familiar syntax while compiling efficiently to native code through C transpilation.