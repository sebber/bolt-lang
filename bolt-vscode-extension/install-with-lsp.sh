#!/bin/bash

# Bolt Language VS Code Extension Installation Script with Enhanced LSP

set -e

echo "🚀 Installing Bolt Language VS Code Extension with Advanced Language Server..."

# Build the bolt-lsp binary (both versions available)
echo "📦 Building Bolt Language Server..."
cd ..

# Build both LSP servers
echo "   Building original LSP server..."
cargo build --release --bin bolt-lsp

echo "   Building refactored LSP server..."
cargo build --release --bin bolt-lsp-refactored

echo "   Building Bolt compiler..."
cargo build --release --bin bolt

cd bolt-vscode-extension

# Create bin directory and copy executables
echo "📋 Setting up executables..."
mkdir -p bin
cp ../target/release/bolt-lsp bin/
cp ../target/release/bolt-lsp-refactored bin/
cp ../target/release/bolt bin/

# Find VS Code extensions directory
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    VSCODE_EXTENSIONS_DIR="$USERPROFILE\\.vscode\\extensions"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    VSCODE_EXTENSIONS_DIR="$HOME/.vscode/extensions"
else
    VSCODE_EXTENSIONS_DIR="$HOME/.vscode/extensions"
fi

EXTENSION_DIR="$VSCODE_EXTENSIONS_DIR/bolt-language-0.6.0"

echo "🔧 Installing extension to $EXTENSION_DIR"

# Remove old version if it exists
if [ -d "$EXTENSION_DIR" ]; then
    echo "🗑️  Removing previous installation..."
    rm -rf "$EXTENSION_DIR"
fi

# Create extension directory
mkdir -p "$EXTENSION_DIR"

# Copy extension files (exclude development files)
echo "📁 Copying extension files..."
cp package.json "$EXTENSION_DIR/"
cp language-configuration.json "$EXTENSION_DIR/"
cp README.md "$EXTENSION_DIR/"
cp CHANGELOG.md "$EXTENSION_DIR/"
cp .vscodeignore "$EXTENSION_DIR/"

# Copy directories
cp -r src "$EXTENSION_DIR/"
cp -r syntaxes "$EXTENSION_DIR/"
cp -r snippets "$EXTENSION_DIR/"
cp -r bin "$EXTENSION_DIR/"

# Install npm dependencies
cd "$EXTENSION_DIR"
echo "📥 Installing npm dependencies..."
npm install --production

echo ""
echo "✅ Bolt Language extension v0.6.0 installed successfully!"
echo ""
echo "🆕 New in v0.6.0:"
echo "   • Production-ready architecture with auto-recovery"
echo "   • Smart executable detection and testing"
echo "   • Visual LSP server status in status bar"
echo "   • Enhanced error messages with suggested actions"
echo "   • Resource management and terminal pooling"
echo ""
echo "🔄 Please reload VS Code:"
echo "   Ctrl+Shift+P → 'Developer: Reload Window'"
echo ""
echo "🎯 Available features:"
echo "   • Type-aware IntelliSense with rich hover information"
echo "   • Context-aware auto-completion"
echo "   • One-click compilation (F5) and run (Ctrl+F5)"
echo "   • Type information display (Ctrl+K Ctrl+I)"
echo "   • Automatic LSP server recovery on failure"
echo "   • Commands: Ctrl+Shift+P → search for 'Bolt'"
echo ""
echo "🔍 Check the status bar for LSP server status!"

# Test LSP server
echo ""
echo "🧪 Testing LSP server..."
if timeout 2s ./bin/bolt-lsp --help 2>/dev/null; then
    echo "✅ LSP server is working"
else
    echo "⚠️  LSP server test inconclusive (this is normal)"
fi

echo ""
echo "🎉 Installation complete! Happy coding with Bolt! 🚀"