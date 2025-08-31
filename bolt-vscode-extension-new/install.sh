#!/bin/bash
set -e

echo "🚀 Installing Bolt Language VS Code Extension v1.0.0"
echo ""

# Clean up old installations
echo "🧹 Cleaning up old Bolt extensions..."
rm -rf ~/.vscode/extensions/bolt-language-*

# Set installation directory
EXTENSION_DIR="$HOME/.vscode/extensions/bolt-language-1.0.0"

echo "📦 Installing to: $EXTENSION_DIR"
mkdir -p "$EXTENSION_DIR"

# Copy all extension files
echo "📋 Copying extension files..."
cp -r ./* "$EXTENSION_DIR/" 2>/dev/null || true

# Ensure LSP binaries are available
echo "🔧 Setting up LSP server..."
if [ ! -f "$EXTENSION_DIR/bin/bolt-lsp" ]; then
    # Try to find and copy bolt-lsp
    if [ -f "../target/release/bolt-lsp" ]; then
        mkdir -p "$EXTENSION_DIR/bin"
        cp ../target/release/bolt-lsp "$EXTENSION_DIR/bin/"
        echo "   ✓ Copied LSP server from release build"
    elif [ -f "../target/debug/bolt-lsp" ]; then
        mkdir -p "$EXTENSION_DIR/bin"
        cp ../target/debug/bolt-lsp "$EXTENSION_DIR/bin/"
        echo "   ✓ Copied LSP server from debug build"
    elif [ -f "../bin/bolt-lsp" ]; then
        mkdir -p "$EXTENSION_DIR/bin"
        cp ../bin/bolt-lsp "$EXTENSION_DIR/bin/"
        echo "   ✓ Copied LSP server from bin directory"
    else
        echo "   ⚠️  LSP server not found - you may need to configure the path in VS Code settings"
    fi
fi

# Also copy bolt compiler if available
if [ ! -f "$EXTENSION_DIR/bin/bolt" ]; then
    if [ -f "../target/release/bolt" ]; then
        cp ../target/release/bolt "$EXTENSION_DIR/bin/" 2>/dev/null || true
        echo "   ✓ Copied Bolt compiler"
    elif [ -f "../target/debug/bolt" ]; then
        cp ../target/debug/bolt "$EXTENSION_DIR/bin/" 2>/dev/null || true
        echo "   ✓ Copied Bolt compiler"
    elif [ -f "../bin/bolt" ]; then
        cp ../bin/bolt "$EXTENSION_DIR/bin/" 2>/dev/null || true
        echo "   ✓ Copied Bolt compiler"
    fi
fi

# Make binaries executable
chmod +x "$EXTENSION_DIR/bin/"* 2>/dev/null || true

echo ""
echo "✅ Extension installed successfully!"
echo ""
echo "📝 Next steps:"
echo "1. Close all VS Code windows"
echo "2. Open VS Code fresh"
echo "3. Open any .bolt file to activate the extension"
echo "4. Check the status bar (bottom right) for 'Bolt LSP Ready'"
echo ""
echo "🎯 Features:"
echo "• Syntax highlighting for .bolt files"
echo "• Language Server Protocol support (hover, completion, diagnostics)"
echo "• Compile with F5, Run with Ctrl+F5"
echo "• Code snippets (type 'fun', 'val', 'var', etc.)"
echo ""
echo "💡 Troubleshooting:"
echo "• If extension doesn't appear: Ctrl+Shift+P → 'Developer: Reload Window'"
echo "• If LSP fails: Check 'Bolt Language Server' output channel"
echo "• Configure LSP path: Settings → Extensions → Bolt → LSP Server Path"