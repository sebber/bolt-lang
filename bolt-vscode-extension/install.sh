#!/bin/bash
# Install Bolt Language Extension for VS Code

EXTENSION_DIR="$HOME/.vscode/extensions/bolt-language-0.1.0"

echo "🔧 Installing Bolt Language Extension for VS Code..."

# Create extension directory
mkdir -p "$EXTENSION_DIR"

# Copy extension files
cp -r . "$EXTENSION_DIR/"

echo "✅ Extension installed to: $EXTENSION_DIR"
echo ""
echo "📝 Next steps:"
echo "1. Reload VS Code (Ctrl+Shift+P → 'Developer: Reload Window')"
echo "2. Open a .bolt file to see syntax highlighting"
echo "3. Try the example.bolt file in the extension folder"
echo ""
echo "🎉 Happy coding with Bolt!"