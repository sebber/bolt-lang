# Bolt Language Extension for Visual Studio Code

**Production-ready IDE support** for the Bolt programming language with advanced type-aware IntelliSense, robust error recovery, and enterprise-grade reliability.

## ✨ Features

### **🚀 Production-Ready Architecture** (New in v0.6.0)
- **Automatic Error Recovery** - LSP server restarts automatically on failure (max 3 attempts)
- **Smart Executable Detection** - Tests compilers and LSP servers before use
- **Visual Status Indicators** - Real-time LSP server status in status bar
- **Resource Management** - Bounded terminal creation with automatic cleanup
- **Enhanced Error Messages** - Contextual error information with suggested actions

### **🧠 Type-Aware IntelliSense** 
- **Rich Hover Information** - Function signatures, parameter types, and return types
- **Context-Aware Auto-completion** - Intelligent suggestions based on actual code analysis
- **Type Information Display** - View variable and function types at cursor position (`Ctrl+K Ctrl+I`)
- **Real-time Type Checking** - Catch type mismatches as you code

### **⚡ Developer Productivity**
- **One-Click Compilation** - Compile current file directly from VS Code (`F5`)
- **Integrated Testing** - Compile and run with single command (`Ctrl+F5`)
- **Enhanced Snippets** - Type-safe code generation with proper annotations
- **Terminal Integration** - Smart terminal pooling for build and execution

### **Language Support**
- **Syntax Highlighting** for `.bolt` files with enhanced grammar
- **Bracket Matching** and auto-closing
- **Comment Support** with `//`
- **Indentation Rules** for code blocks

## Supported Language Features

- **Keywords**: `val`, `var`, `fun`, `def`, `if`, `else`, `for`, `in`, `return`, `import`, `export`
- **Types**: `Integer`, `String`, `Bool`, `Array`, pointer types (`^Type`)
- **Operators**: Arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `!=`, `<`, `>`), logical (`&&`, `||`, `!`), pointers (`&`, `^`)
- **Literals**: Numbers, strings, booleans (`true`, `false`)
- **Functions**: Function calls and definitions
- **Imports**: Module imports with `"bolt:stdio"` style paths

## Example

```bolt
import { print } from "bolt:stdio"

fun add(a: Integer, b: Integer): Integer {
    return a + b
}

val x := 42
val ptr: ^Integer = &x
val value := ptr^

print("Result:")
print(add(value, 10))
```

## Installation

### Option 1: With Language Server (Recommended)

1. Run the installation script:
   ```bash
   ./install-with-lsp.sh
   ```

2. Reload VS Code (Ctrl+Shift+P → "Developer: Reload Window")

3. Open any `.bolt` file - you should see "Bolt Language Server is ready!" message

### Option 2: Manual Installation

1. Build the Bolt Language Server:
   ```bash
   cargo build --release --bin bolt-lsp
   ```

2. Copy this extension to your VS Code extensions folder:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\bolt-language-lsp-0.2.0`
   - **macOS**: `~/.vscode/extensions/bolt-language-lsp-0.2.0`
   - **Linux**: `~/.vscode/extensions/bolt-language-lsp-0.2.0`

3. Install Node.js dependencies:
   ```bash
   cd ~/.vscode/extensions/bolt-language-lsp-0.2.0
   npm install
   ```

4. Reload VS Code

## 🚀 Usage

### **Getting Started**
1. Open any `.bolt` file
2. Start typing `val`, `var`, `fun`, etc. for intelligent auto-completion
3. Hover over functions and variables to see type information
4. Syntax errors and type mismatches will be highlighted in real-time

### **Available Commands** (Ctrl+Shift+P)
- **"Bolt: Compile Current File"** - Compile the active .bolt file (`F5`)
- **"Bolt: Compile and Run Current File"** - Build and execute in one step (`Ctrl+F5`)
- **"Bolt: Show Type Information"** - Display type info for symbol at cursor (`Ctrl+K Ctrl+I`)
- **"Bolt: Restart Bolt Language Server"** - Restart LSP for troubleshooting
- **"Bolt: Show Bolt LSP Output"** - View LSP debug information

### **Keyboard Shortcuts**
- `F5` - Compile current file
- `Ctrl+F5` - Compile and run current file  
- `Ctrl+K Ctrl+I` - Show type information for symbol at cursor

### **Enhanced Snippets**
- `valtype` - Type-safe immutable variable declaration
- `vartype` - Type-safe mutable variable declaration  
- `funtyped` - Function with full type annotations
- `valinfer` - Variable with automatic type inference
- `arraytype` - Typed array declaration
- `errorhandle` - Error handling pattern
- `importall` - Comprehensive module imports
- `fileprocess` - Complete file processing workflow

## About Bolt

Bolt is a simple, powerful programming language with TypeScript-inspired syntax that compiles to native code via C transpilation.

Learn more at: https://github.com/your-username/boltlang