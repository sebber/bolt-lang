#!/bin/bash
# Install Bolt Language Extension with LSP Support for VS Code

set -e

echo "🔧 Building Bolt Language Server..."

# Build the LSP server
cd ..
cargo build --release --bin bolt-lsp
cd bolt-vscode-extension

echo "📦 Installing VS Code extension with LSP..."

EXTENSION_DIR="$HOME/.vscode/extensions/bolt-language-lsp-0.2.0"

# Create extension directory
mkdir -p "$EXTENSION_DIR"

# Copy extension files
cp -r . "$EXTENSION_DIR/"

# Copy the LSP binary to the extension
echo "📦 Copying LSP binary to extension..."
mkdir -p "$EXTENSION_DIR/bin"
cp ../target/release/bolt-lsp "$EXTENSION_DIR/bin/"

# Install dependencies for the extension
if command -v npm &> /dev/null; then
    echo "📥 Installing Node.js dependencies..."
    cd "$EXTENSION_DIR"
    # Clean install to avoid conflicts
    rm -f package-lock.json
    rm -rf node_modules
    npm install --production=false
    cd -
else
    echo "⚠️  npm not found. You may need to install dependencies manually:"
    echo "   cd $EXTENSION_DIR && npm install"
fi

echo "✅ Extension installed to: $EXTENSION_DIR"
echo ""
echo "📝 Next steps:"
echo "1. Reload VS Code (Ctrl+Shift+P → 'Developer: Reload Window')"
echo "2. Open a .bolt file"
echo "3. You should see 'Bolt Language Server is ready!' message"
echo "4. Try typing 'val' or 'fun' for auto-completion"
echo "5. Use Ctrl+Shift+P → 'Bolt: Show Bolt LSP Output' to see LSP logs"
echo ""
echo "🎉 Happy coding with Bolt and LSP support!"
echo ""
echo "💡 Features available:"
echo "   • Real-time syntax error detection"
echo "   • Auto-completion with snippets"
echo "   • Hover information"
echo "   • Import warnings"
echo "   • Command palette integration"