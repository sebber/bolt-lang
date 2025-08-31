#!/bin/bash

# Force VS Code to refresh and recognize the latest Bolt extension

echo "🔄 Forcing VS Code to refresh extension cache..."

# Kill any running VS Code instances
echo "🛑 Stopping VS Code processes..."
pkill -f "code" || true
sleep 2

# Clear all VS Code extension cache locations
echo "🧹 Clearing VS Code extension caches..."

# Main extension cache
rm -rf ~/.vscode/extensions/.obsolete 2>/dev/null || true
rm -rf ~/.vscode/CachedExtensions* 2>/dev/null || true

# Workspace state caches
rm -rf ~/.vscode/User/workspaceStorage/*/state.vscdb* 2>/dev/null || true
rm -rf ~/.vscode/User/globalStorage/state.vscdb* 2>/dev/null || true

# Extension host cache
rm -rf ~/.vscode/extensions/extensions.json 2>/dev/null || true
rm -rf ~/.vscode/User/extensions.json 2>/dev/null || true

# Clear any logs that might contain cached extension info
rm -rf ~/.vscode/logs/* 2>/dev/null || true

# Force rebuild extension registry
echo "🔨 Forcing extension registry rebuild..."
touch ~/.vscode/extensions/bolt-language-0.6.0/.rebuild

echo "✅ Cache clearing complete!"
echo ""
echo "📋 Current Bolt extensions installed:"
ls -la ~/.vscode/extensions/ | grep bolt || echo "   No Bolt extensions found"
echo ""
echo "🚀 Next steps:"
echo "1. Start VS Code: code"
echo "2. Open Command Palette: Ctrl+Shift+P"
echo "3. Run: 'Developer: Reload Window'"
echo "4. Check Extensions view to confirm Bolt Language 0.6.0 is active"
echo "5. Open a .bolt file and check status bar for 'Bolt LSP Ready'"
echo ""
echo "💡 If VS Code still shows 0.4.0:"
echo "   • Extensions view → Bolt Language → Disable → Enable"
echo "   • Or uninstall and reinstall from Extensions marketplace"