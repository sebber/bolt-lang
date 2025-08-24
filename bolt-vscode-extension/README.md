# Bolt Language Extension for Visual Studio Code

Full-featured IDE support for the Bolt programming language with Language Server Protocol (LSP).

## Features

- **Language Server Protocol (LSP)** integration for full IDE experience
- **Real-time Error Detection** - Shows syntax errors as you type
- **Intelligent Auto-completion** - Context-aware suggestions with snippets
- **Hover Information** - Documentation and type information on hover
- **Import Warnings** - Warns when using functions without proper imports
- **Syntax Highlighting** for `.bolt` files
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

## Usage

- Open any `.bolt` file
- Start typing `val`, `var`, `fun`, etc. for auto-completion
- Hover over keywords for documentation
- Syntax errors will be highlighted in real-time
- Use Command Palette (Ctrl+Shift+P) for Bolt commands:
  - "Bolt: Restart Bolt Language Server"
  - "Bolt: Show Bolt LSP Output"

## About Bolt

Bolt is a simple, powerful programming language with TypeScript-inspired syntax that compiles to native code via C transpilation.

Learn more at: https://github.com/your-username/boltlang