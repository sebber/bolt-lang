# Hybrid Error Handling Design for Bolt

## Philosophy
**Explicit, composable, practical** - No exceptions, no silent failures, no "type masturbation"

## Core Design

### 1. **Simple Union Types for Results**
```bolt
// Built-in error handling types
type Result<T, E> = Success<T> | Failure<E>

// Ergonomic constructors
fun success<T>(value: T): Success<T> = Success(value)
fun failure<E>(error: E): Failure<E> = Failure(error) 

// Example usage
fun divide(a: Integer, b: Integer): Result<Integer, String> {
    if (b == 0) {
        return failure("Division by zero")
    }
    return success(a / b)
}
```

### 2. **Pattern Matching Integration**
```bolt
// Pattern match on results
val result := divide(10, 2)
match result {
    Success(value) => print("Result: " + toString(value)),
    Failure(error) => print("Error: " + error)
}

// Elixir-style tuple destructuring also supported
match result {
    {:ok, value} => print("Got: " + toString(value)),
    {:error, reason} => print("Failed: " + reason)
}
```

### 3. **Practical Convenience Methods**
```bolt
// Chain operations that might fail
val result := readFile("data.txt")
    .andThen(parseJson)
    .andThen(extractField("name"))
    .unwrapOr("default_name")

// Early return for errors
fun processData(): Result<String, FileError> {
    val content := readFile("input.txt")?  // ? operator for early return
    val parsed := parseData(content)?
    val result := transform(parsed)?
    return success(result)
}
```

### 4. **Standard Error Types**
```bolt
// Common error types in standard library  
type FileError = {
    kind: FileErrorKind,
    message: String,
    path: String
}

type ParseError = {
    message: String,
    line: Integer,
    column: Integer
}

type NetworkError = {
    code: Integer,
    message: String,
    url: String
}
```

### 5. **Library Integration**
```bolt
// File operations return Results
import { readFile, writeFile } from "bolt:io"

fun processFile(path: String): Result<String, FileError> {
    val content := readFile(path)?
    val processed := processContent(content)
    writeFile("output.txt", processed)?
    return success("File processed successfully")
}

// HTTP operations
import { httpGet, httpPost } from "bolt:http"

fun fetchUserData(id: Integer): Result<User, NetworkError> {
    val response := httpGet("https://api.example.com/users/" + toString(id))?
    val user := parseUser(response.body)?
    return success(user)
}
```

## Advantages

### **Practical Benefits**
1. **No exceptions** - All errors are explicit in type signatures
2. **Composable** - Chain operations with `andThen`, `map`, `?` operator  
3. **Pattern matchable** - Works great with Bolt's pattern matching
4. **Ergonomic** - Not verbose, easy to read and write
5. **Familiar** - Borrows good ideas from Rust and Elixir

### **Type System Integration**
- Union types enable `Success<T> | Failure<E>`
- Pattern matching makes error handling natural
- Type inference reduces boilerplate
- Standard error types provide consistency

### **Performance**
- No exception unwinding overhead
- Compile-time optimizations possible
- Zero-cost abstractions

## Example: File Processing Pipeline
```bolt
import { print } from "bolt:stdio"
import { readFile, writeFile } from "bolt:io"
import { parseJson, stringify } from "bolt:json"

type ProcessingError = FileError | ParseError | ValidationError

fun processConfigFile(inputPath: String, outputPath: String): Result<Void, ProcessingError> {
    // Chain operations - early return on any failure
    val content := readFile(inputPath)?
    val config := parseJson(content)?
    val validated := validateConfig(config)?
    val updated := updateConfig(validated)
    val serialized := stringify(updated)
    writeFile(outputPath, serialized)?
    
    return success(void)
}

// Usage with pattern matching
match processConfigFile("config.json", "config.out.json") {
    Success(_) => print("Config processed successfully"),
    Failure(FileError(err)) => print("File error: " + err.message),
    Failure(ParseError(err)) => print("Parse error at line " + toString(err.line)),
    Failure(ValidationError(err)) => print("Validation failed: " + err.message)
}
```

## Compared to Other Approaches

| Approach | Pros | Cons |
|----------|------|------|
| **Exceptions** | Familiar, clean happy path | Hidden control flow, performance overhead |
| **Go-style (T, error)** | Simple, explicit | Verbose, easy to ignore errors |
| **Rust Result<T, E>** | Composable, type-safe | Can be verbose, learning curve |
| **Elixir {:ok, val} \| {:error, reason}** | Pattern-matchable, explicit | Dynamic typing, runtime overhead |
| **Our Hybrid** | **Best of all worlds** | Need to implement pattern matching |

## Implementation Priority
1. ✅ Union types (`T | E`)
2. ⚙️ Result<T, E> built-in type
3. ⚙️ Pattern matching on unions
4. ⚙️ Standard error types
5. ⚙️ Convenience methods (andThen, ?, unwrapOr)
6. ⚙️ Standard library integration

This gives us **explicit, composable error handling** without being overly complex or academic!