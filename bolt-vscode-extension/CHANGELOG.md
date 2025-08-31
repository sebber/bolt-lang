# Change Log

## [0.6.0] - 2025-08-25

### Major Refactoring - Production-Ready Architecture

#### **Added**
- **Professional Extension Architecture**: Complete refactor with class-based design
- **Smart Executable Detection**: Tests LSP server and compiler before use with comprehensive fallbacks
- **Automatic Error Recovery**: LSP client restart with exponential backoff (max 3 attempts)  
- **Resource Management**: Bounded terminal creation with automatic cleanup
- **Enhanced Logging**: Structured logging with timestamps and multiple levels
- **Status Bar Integration**: Visual LSP server status with loading indicators
- **Configuration Management**: Centralized config with multiple executable search paths
- **Terminal Pooling**: Reuse terminals instead of creating unlimited new ones

#### **Enhanced**
- **Error Messages**: Contextual error information with suggested user actions
- **Path Detection**: Robust executable discovery across development and production environments
- **Command Reliability**: Better error handling for all commands with user feedback
- **LSP Communication**: Middleware for diagnostic filtering and better error recovery

#### **Fixed**
- **Extension Metadata**: Correct repository URLs, license, and publishing configuration
- **Dependency Management**: Moved runtime dependencies from devDependencies
- **Resource Leaks**: Proper cleanup of terminals, channels, and LSP clients
- **Startup Reliability**: Graceful handling of missing executables and failed connections

#### **Developer Experience**
- **Visual Feedback**: Status bar shows LSP server state with loading/ready/error indicators
- **Better Error Reporting**: Specific error messages with resolution suggestions
- **Automatic Recovery**: Failed LSP servers restart automatically without user intervention
- **Enhanced Commands**: All commands now have proper error handling and user feedback

## [0.5.0] - 2025-08-25

### Added
- **Type-Aware IntelliSense**: Enhanced hover information with function signatures and variable types
- **Improved Auto-Completion**: Context-aware completions using actual code analysis from type checker
- **New Commands**:
  - `bolt.compileFile`: Compile current Bolt file
  - `bolt.runFile`: Compile and run current Bolt file in terminal
  - `bolt.showTypeInfo`: Display type information for symbol at cursor
- **Enhanced Snippets**:
  - Type-safe variable declarations (`valtype`, `vartype`)
  - Function with full type annotations (`funtyped`) 
  - Type inference examples (`valinfer`)
  - Typed array declarations (`arraytype`)
  - Error handling patterns (`errorhandle`)
  - Comprehensive import examples (`importall`)
  - Complete file processing workflow (`fileprocess`)

### Enhanced
- LSP initialization with type-aware capabilities
- Updated package description and keywords for better discoverability
- Improved terminal integration for compilation and execution

### Fixed
- Better error handling for LSP communication
- More robust path detection for bolt-lsp executable

## [0.4.0] - Previous Version

### Added
- LSP integration with Language Server Protocol
- Real-time hover information and code completion
- Built-in function documentation
- Standard library module support (bolt:stdio, bolt:math, bolt:io, bolt:string)
- Commands for LSP management (restart, show output)
- Generic type and native function snippets
- File I/O and string processing examples

## [0.1.0] - 2024-08-24

### Added
- Initial release
- Syntax highlighting for Bolt language
- Support for keywords: `val`, `var`, `fun`, `def`, `if`, `else`, `for`, `in`, `return`, `import`, `export`
- Support for primitive types: `Integer`, `String`, `Bool`
- Support for pointer syntax: `^Type`, `&variable`, `pointer^`
- Support for operators: arithmetic, comparison, logical, assignment
- Support for literals: numbers, strings, booleans
- Function call highlighting
- Module import highlighting
- Auto-closing brackets and quotes
- Indentation rules
- Line comments with `//`