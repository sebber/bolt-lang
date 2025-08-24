#!/bin/bash
echo "🔄 Completely reinstalling Bolt VS Code extension..."

# Kill any running VS Code processes to free up the LSP binary
echo "📦 Stopping VS Code processes..."
pkill -f "code" || true
sleep 2

# Remove old extension completely
echo "🗑️  Removing old extension..."
rm -rf ~/.vscode/extensions/bolt-language-lsp-*
rm -rf ~/.vscode/extensions/bolt-language-*

# Build fresh LSP
echo "🔨 Building fresh LSP binary..."
cargo build --bin bolt-lsp

# Install extension cleanly
echo "📦 Installing fresh extension..."
cd bolt-vscode-extension
./install-with-lsp.sh

echo ""
echo "✅ Fresh installation complete!"
echo ""
echo "📝 Next steps:"
echo "1. Start VS Code"
echo "2. Open a .bolt file"
echo "3. Check View → Output → 'Bolt Language Server' for messages"
echo "4. Try hovering - should see verbose debug output"